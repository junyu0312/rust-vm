use std::sync::Arc;

use crate::device::Result;
use crate::device::mmio::MmioLayout;

pub trait DeviceVmExitHandler {
    fn io_in(&mut self, port: u16, data: &mut [u8]) -> Result<()>;
    fn io_out(&mut self, port: u16, data: &[u8]) -> Result<()>;
    fn mmio_read(&self, addr: u64, len: usize, data: &mut [u8]) -> Result<()>;
    fn mmio_write(&self, addr: u64, len: usize, data: &[u8]) -> Result<()>;
    fn in_mmio_range(&self, addr: u64) -> bool;
    fn mmio_layout(&self) -> Arc<MmioLayout>;
}
