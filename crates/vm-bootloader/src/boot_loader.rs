use std::path::PathBuf;
use std::slice::Iter;

use vm_core::device::Device;
use vm_core::layout;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::virt::Virt;

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
    LayoutError(layout::Error),
    #[error("{0}")]
    GenerateDtb(vm_fdt::Error),
}

impl From<layout::Error> for Error {
    fn from(err: layout::Error) -> Self {
        Error::LayoutError(err)
    }
}

impl From<vm_fdt::Error> for Error {
    fn from(err: vm_fdt::Error) -> Self {
        Error::GenerateDtb(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait BootLoaderBuilder<V>
where
    V: Virt,
    Self: BootLoader<V>,
{
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self;
}

pub trait BootLoader<V>
where
    V: Virt,
{
    fn load(
        &self,
        virt: &mut V,
        memory: &mut MemoryAddressSpace<V::Memory>,
        irq_chip: &V::Irq,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<()>;
}
