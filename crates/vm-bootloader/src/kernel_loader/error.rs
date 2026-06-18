use thiserror::Error;
use vm_firmware::acpi::error::AcpiError;
use vm_utils::range_allocator::RangeAllocatorError;

#[derive(Error, Debug)]
pub enum KernelLoaderError {
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

    #[error("Failed to reserve ram, err: {0}")]
    ReserveRam(#[from] RangeAllocatorError),

    #[error("initramfs too large")]
    InitramfsTooLarge,

    #[error("initramfs address too high")]
    InitramfsAddressTooHigh,

    #[error("Cmdline too large")]
    CmdlineTooLarge,

    #[cfg(target_arch = "x86_64")]
    #[error("Copy cmdline into memory failed")]
    CopyCmdlineFailed,

    #[error("Acpi error: {0}")]
    Acpi(#[from] AcpiError),

    #[cfg(target_arch = "x86_64")]
    #[error("Copy gdt into memory failed")]
    CopyGdtFailed,
}
