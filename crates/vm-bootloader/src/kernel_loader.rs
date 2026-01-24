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
    #[error(
        "Out of memory, kernel_end: 0x{kernel_end:x}, memory_end: 0x{memory_end:x}, memory_base: 0x{memory_base:x}, memory_size: {memory_size}"
    )]
    OutOfMemory {
        kernel_end: u64,
        memory_end: u64,
        memory_base: u64,
        memory_size: u64,
    },
    #[cfg(target_arch = "x86_64")]
    #[error("Copy cmdline into memory failed")]
    CopyCmdlineFailed,
}

pub struct LoadResult {
    pub start_pc: u64,
    pub kernel_start: u64,
    pub kernel_end: u64,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait KernelLoader<C> {
    type BootParams;

    fn load(
        &self,
        boot_params: &Self::BootParams,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<LoadResult>;
}
