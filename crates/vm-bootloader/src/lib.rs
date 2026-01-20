#![deny(warnings)]

use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::virt::Virt;

pub mod linux;

pub trait BootLoader<V>
where
    V: Virt,
{
    fn install(
        &self,
        memory: &mut MemoryAddressSpace<V::Memory>,
        memory_size: usize,
        vcpu0: &mut V::Vcpu,
    ) -> anyhow::Result<()>;
}
