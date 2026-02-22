use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_core::irq::InterruptController;
use vm_core::irq::Phandle;

use crate::device::gic::gic_common::config::GicConfig;
use crate::device::gic::gic_common::register::gicd::Distributor;
use crate::device::gic::gic_common::register::gicr::RedistributorRegion;

struct GicV3Internal {
    distributor_base: u64,
    distributor_size: u64,
    redistributor_region_base: u64,
    redistributor_region_size: u64,
    distributor: Distributor,
    redistributor_region: RedistributorRegion, // We only support single region
}

pub struct GicV3DistributorHandler {
    internal: Arc<Mutex<GicV3Internal>>,
}

impl MmioHandler for GicV3DistributorHandler {
    fn mmio_range(&self) -> MmioRange {
        let internal = self.internal.lock().unwrap();

        MmioRange {
            start: internal.distributor_base,
            len: internal.distributor_size.try_into().unwrap(),
        }
    }

    fn mmio_read(&self, offset: u64, _len: usize, data: &mut [u8]) {
        let mut internal = self.internal.lock().unwrap();

        internal.distributor.mmio_read(offset, data);
    }

    fn mmio_write(&self, offset: u64, _len: usize, data: &[u8]) {
        let mut internal = self.internal.lock().unwrap();

        internal.distributor.mmio_write(offset, data);
    }
}

pub struct GicV3RedistributorHandler {
    internal: Arc<Mutex<GicV3Internal>>,
}

impl MmioHandler for GicV3RedistributorHandler {
    fn mmio_range(&self) -> MmioRange {
        let internal = self.internal.lock().unwrap();

        MmioRange {
            start: internal.redistributor_region_base,
            len: internal.redistributor_region_size.try_into().unwrap(),
        }
    }

    fn mmio_read(&self, offset: u64, _len: usize, data: &mut [u8]) {
        let mut internal = self.internal.lock().unwrap();

        let redistributor_idx =
            (offset / internal.redistributor_region.redist_stride as u64) as usize;
        let redistributor = internal
            .redistributor_region
            .redistributors
            .get_mut(redistributor_idx)
            .unwrap();

        redistributor.mmio_read(offset, data);
    }

    fn mmio_write(&self, offset: u64, _len: usize, data: &[u8]) {
        let mut internal = self.internal.lock().unwrap();

        let redistributor_idx =
            (offset / internal.redistributor_region.redist_stride as u64) as usize;
        let redistributor = internal
            .redistributor_region
            .redistributors
            .get_mut(redistributor_idx)
            .unwrap();

        redistributor.mmio_write(offset, data);
    }
}

pub struct GicV3Device {
    internal: Arc<Mutex<GicV3Internal>>,
}

impl Device for GicV3Device {
    fn name(&self) -> String {
        "gic-v3".to_string()
    }
}

impl MmioDevice for GicV3Device {
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        let distributor_handler = GicV3DistributorHandler {
            internal: self.internal.clone(),
        };

        let redistributor_handler = GicV3RedistributorHandler {
            internal: self.internal.clone(),
        };

        vec![
            Box::new(distributor_handler),
            Box::new(redistributor_handler),
        ]
    }

    fn generate_dt(&self, _fdt: &mut vm_fdt::FdtWriter) -> Result<(), vm_fdt::Error> {
        // skip
        Ok(())
    }
}

pub struct GicV3 {
    internal: Arc<Mutex<GicV3Internal>>,
}

impl GicV3 {
    pub fn new(config: GicConfig) -> Self {
        GicV3 {
            internal: Arc::new(Mutex::new(GicV3Internal {
                distributor_base: config.distributor_base,
                distributor_size: 0x10000,
                redistributor_region_base: config.redistributor_base,
                redistributor_region_size: 0x200000,
                distributor: Distributor::new(&config),
                redistributor_region: RedistributorRegion::new(&config),
            })),
        }
    }

    pub fn get_device(&self) -> Box<GicV3Device> {
        Box::new(GicV3Device {
            internal: self.internal.clone(),
        })
    }
}

impl InterruptController for GicV3 {
    fn trigger_irq(&self, _irq_line: u32, _active: bool) {
        todo!()
    }

    fn send_msi(&self, _intid: u32) {
        todo!()
    }

    fn write_device_tree(
        &self,
        fdt: &mut vm_fdt::FdtWriter,
    ) -> anyhow::Result<vm_core::irq::Phandle> {
        let internal = self.internal.lock().unwrap();

        let gic_node = fdt.begin_node(&format!(
            "interrupt-controller@{:016x}",
            internal.distributor_base
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
                internal.distributor_base,
                internal.distributor_size,
                internal.redistributor_region_base,
                internal.redistributor_region_size,
            ],
        )?;
        fdt.property_null("ranges")?;

        if true {
            fdt.property_null("msi-controller")?;
            fdt.property_array_u32("mbi-ranges", &[128, 128])?;
        }

        fdt.end_node(gic_node)?;

        Ok(Phandle::GIC)
    }
}
