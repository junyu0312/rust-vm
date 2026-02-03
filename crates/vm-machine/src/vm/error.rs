use vm_bootloader::boot_loader::Error as BootLoaderError;
use vm_core::virt::error::VirtError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Platform error: {0}")]
    Platform(VirtError),

    #[error("Failed to init cpu, error: {0}")]
    InitCpu(String),

    #[error("Failed to init memory, error: {0}")]
    InitMemory(String),

    #[error("Failed to init irqchip, error: {0}")]
    InitIrqchip(String),

    #[error("Failed to post init, error: {0}")]
    PostInit(String),

    #[error("Failed to init device, error: {0}")]
    InitDevice(String),

    #[error("Failed to setup with bootloader, error: {0}")]
    Bootloader(BootLoaderError),

    #[error("Failed to run vm, error: {0}")]
    Run(String),
}

impl From<VirtError> for Error {
    fn from(err: VirtError) -> Self {
        Error::Platform(err)
    }
}

impl From<vm_bootloader::boot_loader::Error> for Error {
    fn from(err: vm_bootloader::boot_loader::Error) -> Self {
        Error::Bootloader(err)
    }
}
