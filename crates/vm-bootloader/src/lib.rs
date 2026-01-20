use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;

pub mod linux;

pub trait BootLoader<V> {
    fn init<C>(
        &self,
        memory: &mut MemoryAddressSpace<C>,
        memory_size: usize,
        vcpu0: &mut V,
    ) -> anyhow::Result<()>
    where
        C: MemoryContainer;
}
