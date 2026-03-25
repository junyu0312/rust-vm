use vm_core::debug::gdbstub::GdbStubError;
use vm_core::utils::address_space::AddressSpaceError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Platform error: {0}")]
    Platform(#[from] vm_core::error::Error),

    #[error("No irq_chip is specified")]
    NoIrqChipSpecified,

    #[error("Device address space error: {0}")]
    DeviceAddressSpace(#[from] AddressSpaceError),

    #[error("Pci device error: {0}")]
    PciDevice(#[from] vm_pci::error::Error),

    #[error("Failed to init memory, error: {0}")]
    InitMemory(String),

    #[error("Failed to init irqchip, error: {0}")]
    InitIrqchip(String),

    #[error("Failed to setup with bootloader, error: {0}")]
    Bootloader(#[from] vm_bootloader::boot_loader::Error),

    #[error("gdb_stub error: {0}")]
    GdbStub(#[from] GdbStubError),

    #[error("monitor error: {0}")]
    Monitor(#[from] vm_core::monitor::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
