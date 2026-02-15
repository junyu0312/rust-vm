use std::sync::Arc;

use applevisor::gic::GicConfig;
use applevisor::vm::GicEnabled;
use applevisor::vm::VirtualMachineInstance;
use tracing::warn;

use crate::irq::InterruptController;
use crate::irq::Phandle;
use crate::irq::arch::aarch64::AArch64IrqChip;

pub struct HvpGicV3 {
    distributor_base: u64,
    redistributor_base: u64,
    msi_base: u64,
    vm: Arc<VirtualMachineInstance<GicEnabled>>,
}

impl HvpGicV3 {
    pub fn new(
        distributor_base: u64,
        redistributor_base: u64,
        msi_base: u64,
        vm: Arc<VirtualMachineInstance<GicEnabled>>,
    ) -> Self {
        HvpGicV3 {
            distributor_base,
            redistributor_base,
            msi_base,
            vm,
        }
    }
}

impl InterruptController for HvpGicV3 {
    fn trigger_irq(&self, irq_line: u32, active: bool) {
        // assert!(irq_line >= 32);

        if let Err(err) = self.vm.gic_set_spi(irq_line, active) {
            warn!(irq_line, ?err, "Failed to send spi");
        }
    }

    fn send_msi(&self, intid: u32) {
        self.vm.gic_send_msi(self.msi_base, intid).unwrap();
    }
}

impl AArch64IrqChip for HvpGicV3 {
    fn get_distributor_base(&self) -> u64 {
        self.distributor_base
    }

    fn get_distributor_size(&self) -> anyhow::Result<usize> {
        let size = GicConfig::get_distributor_size()?;
        Ok(size)
    }

    fn get_redistributor_base(&self) -> u64 {
        self.redistributor_base
    }

    fn get_redistributor_region_size(&self) -> anyhow::Result<usize> {
        let size = GicConfig::get_redistributor_region_size()?;
        Ok(size)
    }

    fn get_msi_region_base(&self) -> u64 {
        self.msi_base
    }

    fn get_msi_region_size(&self) -> anyhow::Result<usize> {
        Ok(GicConfig::get_msi_region_size()?)
    }

    fn write_device_tree(&self, fdt: &mut vm_fdt::FdtWriter) -> anyhow::Result<u32> {
        let gic_node = fdt.begin_node(&format!(
            "interrupt-controller@{:016x}",
            self.get_distributor_base()
        ))?;
        fdt.property_string("compatible", "arm,gic-v3")?;
        fdt.property_u32("#interrupt-cells", 3)?;
        fdt.property_null("interrupt-controller")?;
        fdt.property_u32("#address-cells", 2)?;
        fdt.property_u32("#size-cells", 2)?;
        fdt.property_phandle(Phandle::GIC as u32)?;
        fdt.property_array_u64(
            "reg",
            &[
                self.get_distributor_base(),
                self.get_distributor_size()? as u64,
                self.get_redistributor_base(),
                self.get_redistributor_region_size()? as u64,
            ],
        )?;

        fdt.property_null("ranges")?;
        let msi_node = fdt.begin_node(&format!(
            "msi-controller@{:016x}",
            self.get_msi_region_base()
        ))?;
        fdt.property_string("compatible", "arm,gic-v3-its")?;
        fdt.property_null("msi-controller")?;
        // fdt.property_u32("#msi-cells", 1)?;
        fdt.property_array_u64(
            "reg",
            &[
                self.get_msi_region_base(),
                self.get_msi_region_size()? as u64,
            ],
        )?;
        fdt.property_phandle(Phandle::MSI as u32)?;
        fdt.end_node(msi_node)?;

        fdt.end_node(gic_node)?;

        Ok(Phandle::GIC as u32)
    }
}
