#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Platform error: {0}")]
    Platform(vm_core::error::Error),

    #[error("No irq_chip is specified")]
    NoIrqChipSpecified,

    #[error("Device error: {0}")]
    Device(vm_core::device::Error),

    #[error("Pci device error: {0}")]
    PciDevice(vm_pci::error::Error),

    #[error("Failed to init memory, error: {0}")]
    InitMemory(String),

    #[error("Failed to init irqchip, error: {0}")]
    InitIrqchip(String),

    #[error("Failed to setup with bootloader, error: {0}")]
    Bootloader(vm_bootloader::boot_loader::Error),

    #[error("gdb_stub failed, error: {0}")]
    GdbStub(String),
}

impl From<vm_core::error::Error> for Error {
    fn from(err: vm_core::error::Error) -> Self {
        Error::Platform(err)
    }
}

impl From<vm_core::device::Error> for Error {
    fn from(err: vm_core::device::Error) -> Self {
        Error::Device(err)
    }
}

impl From<vm_pci::error::Error> for Error {
    fn from(err: vm_pci::error::Error) -> Self {
        Error::PciDevice(err)
    }
}

impl From<vm_bootloader::boot_loader::Error> for Error {
    fn from(err: vm_bootloader::boot_loader::Error) -> Self {
        Error::Bootloader(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
