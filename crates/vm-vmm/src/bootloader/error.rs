use thiserror::Error;
use vm_core::virtualization::vm::error::VmError;

#[derive(Error, Debug)]
pub enum BootloaderError {
    #[error("{0}")]
    Bootloader(#[from] vm_bootloader::boot_loader::Error),

    #[error("{0}")]
    Vm(#[from] VmError),
}
