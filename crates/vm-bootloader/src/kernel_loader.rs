use thiserror::Error;
#[cfg(target_arch = "x86_64")]
use vm_arch::x86_64::gdt::Gdt;
use vm_mm::manager::MemoryAddressSpace;

pub mod linux;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Read failed")]
    ReadFailed,

    #[error("Invalid kernel image")]
    InvalidKernelImage,

    #[error("Set kernel start offset is not supported")]
    KernelStartOffsetNotSupport,

    #[error("The loaded kernel is too old")]
    KernelTooOld,

    #[error("Invalid address alignment")]
    InvalidAddressAlignment,

    #[error("Copy kernel into memory failed, reason: {0}")]
    CopyKernelFailed(vm_mm::error::Error),

    #[error(
        "Out of memory, kernel_end: 0x{kernel_end:x}, memory_end: 0x{memory_end:x}, memory_base: 0x{memory_base:x}, memory_size: {memory_size}"
    )]
    OutOfMemory {
        kernel_end: u64,
        memory_end: u64,
        memory_base: u64,
        memory_size: u64,
    },

    #[error("initramfs too large")]
    InitramfsTooLarge,

    #[error("initramfs address too high")]
    InitramfsAddressTooHigh,

    #[error("Copy initramfs into memory failed, reason: {0}")]
    CopyInitramfsFailed(vm_mm::error::Error),

    #[error("Cmdline too large")]
    CmdlineTooLarge,

    #[cfg(target_arch = "x86_64")]
    #[error("Copy cmdline into memory failed")]
    CopyCmdlineFailed,

    #[cfg(target_arch = "x86_64")]
    #[error("Copy gdt into memory failed")]
    CopyGdtFailed,
}

pub struct LoadResult {
    pub start_pc: u64,
    pub kernel_start: u64,
    pub kernel_len: usize,
    #[cfg(target_arch = "x86_64")]
    pub gdt: Gdt<5>,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait KernelLoader {
    type BootParams;

    fn load(
        &mut self,
        boot_params: &Self::BootParams,
        memory: &MemoryAddressSpace,
    ) -> Result<LoadResult>;
}
