use vm_core::arch::Arch;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::vcpu::Vcpu;

pub mod arch;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Load dtb failed, reason: {0}")]
    LoadDtbFailed(String),
    #[error("Setup kernel failed, reason: {0}")]
    LoadKernelFailed(String),
    #[error("Setup Boot cpu failed, reason: {0}")]
    SetupBootCpuFailed(String),
    #[error("Memory overlap")]
    MemoryOverlap,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait BootLoader<M, A, V>
where
    M: MemoryContainer,
    A: Arch,
    V: Vcpu<A>,
{
    fn load(
        &self,
        ram_base: u64,
        memory: &mut MemoryAddressSpace<M>,
        vcpus: &mut Vec<V>,
    ) -> Result<()>;
}
