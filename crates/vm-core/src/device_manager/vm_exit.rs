#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("no device found for port 0x{0:#x}")]
    NoDeviceForPort(u16),

    #[error("no device found for addr 0x{0:#x}")]
    NoDeviceForAddr(u64),

    #[error(
        "mmio out of memory: mmio_start: 0x{mmio_start:x}, mmio_len: {mmio_len}, addr: 0x{addr:x}"
    )]
    MmioOutOfMemory {
        mmio_start: u64,
        mmio_len: usize,
        addr: u64,
    },
}

pub trait DeviceVmExitHandler: Send + Sync {
    fn io_in(&mut self, port: u16, data: &mut [u8]) -> Result<(), DeviceError>;
    fn io_out(&mut self, port: u16, data: &[u8]) -> Result<(), DeviceError>;
    fn mmio_read(&self, addr: u64, len: usize, data: &mut [u8]) -> Result<(), DeviceError>;
    fn mmio_write(&self, addr: u64, len: usize, data: &[u8]) -> Result<(), DeviceError>;
    fn in_mmio_region(&self, addr: u64) -> bool;
}
