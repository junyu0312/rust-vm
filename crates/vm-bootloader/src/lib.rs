#![deny(warnings)]

use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::virt::Virt;

pub mod linux;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Read failed")]
    ReadFailed,
    #[error("Invalid kernel image")]
    InvalidKernelImage,
    #[error("Invalid address alignment")]
    InvalidAddressAlignment,
    #[error("Copy kernel into memory failed, reason: {0}")]
    CopyKernelFailed(String),
    #[error("Setup kernel failed")]
    SetupKernelFailed,
    #[error("Setup initrd into memory failed")]
    SetupInitrdFailed,
    #[error("Copy cmdline into memory failed")]
    CopyCmdlineFailed,
    #[error("Setup dtb failed, reason: {0}")]
    SetupDtbFailed(String),
    #[error("Setup bootcpu failed")]
    SetupBootcpuFailed,
    // TODO: Remove it
    #[error("Setup firmware failed")]
    SetupFirmwareFailed,
}

pub trait BootLoader<V>
where
    V: Virt,
{
    fn install(
        &self,
        ram_base: u64,
        memory: &mut MemoryAddressSpace<V::Memory>,
        memory_size: usize,
        vcpu0: &mut V::Vcpu,
    ) -> Result<(), Error>;
}
