use vm_core::mm::manager::MemoryRegions;
use vm_core::vcpu::Vcpu;

pub mod linux;

pub trait BootLoader {
    fn init(
        &self,
        memory: &mut MemoryRegions,
        memory_size: usize,
        vcpu0: &mut dyn Vcpu,
    ) -> anyhow::Result<()>;
}
