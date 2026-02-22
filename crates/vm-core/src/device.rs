use std::fmt::Debug;

pub mod address_space;
pub mod device_manager;
pub mod mmio;
pub mod pio;
pub mod vm_exit;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid length of range")]
    InvalidLen,
    #[error("invalid range, offset: 0x{0:x}, len: {1}")]
    InvalidRange(u64, usize),
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
    #[error("irq_chip already exists")]
    IrqChipAlreadyExists,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Device {
    fn name(&self) -> String;
}
