use std::path::PathBuf;
use std::slice::Iter;

use vm_core::arch::irq::InterruptController;
use vm_core::arch::layout;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_mm::manager::MemoryAddressSpace;

pub mod arch;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Load dtb failed, reason: {0}")]
    LoadDtbFailed(String),
    #[error("Setup kernel failed, reason: {0}")]
    LoadKernelFailed(String),
    #[error("Load initd failed, reason: {0}")]
    LoadInitrdFailed(String),
    #[error("Setup Boot cpu failed, reason: {0}")]
    SetupBootCpuFailed(String),
    #[error("Memory overlap")]
    MemoryOverlap,
    #[error("Layout error, reason: {0}")]
    LayoutError(#[from] layout::Error),
    #[error("{0}")]
    GenerateDtb(#[from] vm_fdt::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait BootLoaderBuilder
where
    Self: BootLoader,
{
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self;
}

pub trait BootLoader {
    fn load(
        &self,
        ram_size: u64,
        vcpus: usize,
        memory: &MemoryAddressSpace,
        irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<u64>;
}
