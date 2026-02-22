use vm_fdt::FdtWriter;

use crate::device::Device;
use crate::device::mmio::MmioRange;

pub trait MmioHandler: Send {
    fn mmio_range(&self) -> MmioRange;

    fn mmio_read(&self, offset: u64, len: usize, data: &mut [u8]);

    fn mmio_write(&self, offset: u64, len: usize, data: &[u8]);
}

pub trait MmioDevice: Device {
    fn mmio_range_handlers(&self) -> Vec<Box<dyn MmioHandler>>;

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error>;
}
