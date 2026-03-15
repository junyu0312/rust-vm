use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::Device;
use vm_core::device::mmio::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::mmio::mmio_device::MmioHandler;
use vm_fdt::FdtWriter;
use vm_mm::memory_container::MemoryContainer;

use crate::device::VirtioDevice;
use crate::transport::VirtioDev;

mod control_register;
mod mmio_handler;

pub struct VirtioMmioTransport<C, D> {
    mmio_range: MmioRange,
    dev: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> Clone for VirtioMmioTransport<C, D> {
    fn clone(&self) -> Self {
        Self {
            mmio_range: self.mmio_range,
            dev: self.dev.clone(),
        }
    }
}

impl<C, D> VirtioMmioTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioDevice<C>,
{
    pub fn new(dev: Arc<Mutex<VirtioDev<C, D>>>, mmio_range: MmioRange) -> Self {
        VirtioMmioTransport { mmio_range, dev }
    }

    pub fn dev(&self) -> Arc<Mutex<VirtioDev<C, D>>> {
        self.dev.clone()
    }
}

impl<C, D> Device for VirtioMmioTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioDevice<C>,
{
    fn name(&self) -> String {
        D::NAME.to_string()
    }
}

impl<C, D> MmioDevice for VirtioMmioTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioDevice<C>,
{
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>> {
        vec![Box::new(self.clone())]
    }

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error> {
        let dev = self.dev.lock().unwrap();

        let node = fdt.begin_node(&format!("{}@{:x}", self.name(), self.mmio_range.start))?;

        fdt.property_string("compatible", "virtio,mmio")?;
        fdt.property_array_u64("reg", &[self.mmio_range.start, self.mmio_range.len as u64])?;
        if let Some(irq) = dev.device.irq() {
            #[cfg(target_arch = "aarch64")]
            {
                use vm_core::arch::aarch64::irq::GIC_SPI;
                use vm_core::arch::aarch64::irq::IRQ_TYPE_LEVEL_HIGH;
                fdt.property_array_u32("interrupts", &[GIC_SPI, irq, IRQ_TYPE_LEVEL_HIGH])?;
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                std::hint::black_box(irq);
                todo!()
            }
        }

        fdt.end_node(node)?;

        Ok(())
    }
}
