use anyhow::anyhow;
use kvm_bindings::kvm_userspace_memory_region;

use crate::kvm::vm::KvmVm;
use crate::mm::MemoryRegion;

impl KvmVm {
    pub fn init_mm(&mut self, memory_gb: usize) -> anyhow::Result<()> {
        let memory_region = MemoryRegion::new(memory_gb)?;

        unsafe {
            self.vm_fd
                .set_user_memory_region(kvm_userspace_memory_region {
                    slot: 0,
                    flags: 0,
                    guest_phys_addr: 0x0,
                    memory_size: (memory_gb as u64) << 30,
                    userspace_addr: memory_region.as_u64(),
                })?;
        }

        self.memory_regions
            .set(vec![memory_region])
            .map_err(|_| anyhow!("memory regions are already set"))?;

        Ok(())
    }
}
