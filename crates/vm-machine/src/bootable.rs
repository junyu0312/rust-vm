use vm_core::mm::manager::MemoryRegions;

use crate::kvm::vcpu::KvmVcpu;

pub mod linux;

pub trait Bootable {
    fn init(
        &self,
        memory: &mut MemoryRegions,
        memory_size: usize,
        vcpu0: &mut KvmVcpu,
    ) -> anyhow::Result<()>;
}
