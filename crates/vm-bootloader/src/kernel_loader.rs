use vm_core::mm::manager::MemoryAddressSpace;

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

pub struct LoadResult {
    pub start_pc: u64,
    pub kernel_start: u64,
    pub kernel_end: u64,
}

pub trait KernelLoader<C> {
    type BootParams;

    fn load(
        &self,
        boot_params: &Self::BootParams,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<LoadResult, Error>;
}
