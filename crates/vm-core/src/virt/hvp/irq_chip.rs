use std::sync::Arc;

use applevisor::gic::GicConfig;
use applevisor::vm::GicEnabled;
use applevisor::vm::VirtualMachineInstance;
use tracing::warn;

use crate::irq::InterruptController;
use crate::irq::arch::aarch64::AArch64IrqChip;

const PHANDLE: u32 = 0x1;

pub struct HvpGicV3 {
    distributor_base: u64,
    redistributor_base: u64,
    vm: Arc<VirtualMachineInstance<GicEnabled>>,
}

impl HvpGicV3 {
    pub fn new(
        distributor_base: u64,
        redistributor_base: u64,
        vm: Arc<VirtualMachineInstance<GicEnabled>>,
    ) -> Self {
        HvpGicV3 {
            distributor_base,
            redistributor_base,
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
}

impl AArch64IrqChip for HvpGicV3 {
    fn get_distributor_base(&self) -> anyhow::Result<u64> {
        Ok(self.distributor_base)
    }

    fn get_distributor_size(&self) -> anyhow::Result<usize> {
        let size = GicConfig::get_distributor_size()?;
        Ok(size)
    }

    fn get_redistributor_base(&self) -> anyhow::Result<u64> {
        Ok(self.redistributor_base)
    }

    fn get_redistributor_region_size(&self) -> anyhow::Result<usize> {
        let size = GicConfig::get_redistributor_region_size()?;
        Ok(size)
    }

    fn write_device_tree(&self, fdt: &mut vm_fdt::FdtWriter) -> anyhow::Result<u32> {
        let gic_node = fdt.begin_node(&format!(
            "interrupt-controller@{:016x}",
            self.get_distributor_base()?
        ))?;
        fdt.property_string("compatible", "arm,gic-v3")?;
        fdt.property_u32("#interrupt-cells", 3)?;
        fdt.property_null("interrupt-controller")?;
        fdt.property_phandle(PHANDLE)?;
        fdt.property_array_u64(
            "reg",
            &[
                self.get_distributor_base()?,
                self.get_distributor_size()? as u64,
                self.get_redistributor_base()?,
                self.get_redistributor_region_size()? as u64,
            ],
        )?;
        fdt.end_node(gic_node)?;

        Ok(PHANDLE)
    }
}
