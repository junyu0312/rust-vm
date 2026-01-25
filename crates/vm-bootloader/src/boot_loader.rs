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
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait BootLoader<V>
where
    V: Virt,
{
    fn load(
        &self,
        ram_size: u64,
        memory: &mut MemoryAddressSpace<V::Memory>,
        vcpus: &mut Vec<V::Vcpu>,
    ) -> Result<()>;
}
