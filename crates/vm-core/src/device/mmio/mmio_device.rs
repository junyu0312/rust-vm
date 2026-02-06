use vm_fdt::FdtWriter;

use crate::device::Device;
use crate::device::mmio::MmioRange;

pub trait MmioDevice: Device {
    fn mmio_range(&self) -> MmioRange;

    fn mmio_read(&mut self, offset: u64, len: usize, data: &mut [u8]);

    fn mmio_write(&mut self, offset: u64, len: usize, data: &[u8]);

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), vm_fdt::Error>;
}
