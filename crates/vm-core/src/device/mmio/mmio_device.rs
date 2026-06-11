use std::ops::Range;

use vm_fdt::FdtWriter;

use crate::device::error::DeviceError;

pub trait MmioDevice {
    fn mmio_ranges(&self) -> Vec<Range<u64>>;

    fn mmio_read(&self, addr: u64, buf: &mut [u8]) -> Result<(), DeviceError>;

    fn mmio_write(&self, addr: u64, buf: &[u8]) -> Result<(), DeviceError>;

    fn generate_dt(&self, fdt: &mut FdtWriter) -> Result<(), DeviceError>;
}
