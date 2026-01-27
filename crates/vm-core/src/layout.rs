#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("arch: {0} already set")]
    ArchAlreadySet(String),
    #[error("arch: {0} unset")]
    ArchUnset(String),

    #[error("ram_size already set")]
    RamSizeAlreadySet,
    #[error("ram_size unset")]
    RamSizeUnset,

    #[error("kernel already set")]
    KernelAlreadySet,
    #[error("kernel unset")]
    KernelUnset,
    #[error("kernel too large")]
    KernelTooLarge,

    #[error("initrd already set")]
    InitrdAlreadySet,
    #[error("initrd unset")]
    InitrdUnset,

    #[error("dtb already set")]
    DtbAlreadySet,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait MemoryLayout {
    fn get_mmio_start(&self) -> u64;
    fn get_mmio_len(&self) -> usize;

    fn get_ram_base(&self) -> u64;
    fn set_ram_size(&self, len: u64) -> Result<()>;
    fn get_ram_size(&self) -> Result<u64>;

    fn set_kernel(&self, kernel_start: u64, kernel_len: usize, start_pc: u64) -> Result<()>;
    fn get_kernel_start(&self) -> Result<u64>;
    fn get_kernel_len(&self) -> Result<usize>;
    fn get_start_pc(&self) -> Result<u64>;

    fn get_initrd_start(&self) -> u64;
    fn set_initrd_len(&self, len: usize) -> Result<()>;
    fn get_initrd_len(&self) -> Result<usize>;

    fn get_dtb_start(&self) -> u64;
    fn set_dtb_len(&self, len: usize) -> Result<()>;

    fn validate(&self) -> Result<()>;
}
