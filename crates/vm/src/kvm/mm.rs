use anyhow::anyhow;
use kvm_bindings::kvm_userspace_memory_region;

use crate::kvm::vm::KvmVm;
use crate::mm::manager::MemoryRegions;
use crate::mm::region::MemoryRegion;

impl KvmVm {
    pub fn init_mm(&mut self, memory_gb: usize) -> anyhow::Result<()> {
        let memory_region = MemoryRegion::new(0, memory_gb << 30)?;

        let mut memory_regions = MemoryRegions::default();
        memory_regions
            .try_insert(memory_region)
            .map_err(|_| anyhow!("Failed to insert memory_region"))?;

        for memory_region in &memory_regions {
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
        }

        self.memory_regions
            .set(memory_regions)
            .map_err(|_| anyhow!("memory regions are already set"))?;

        Ok(())
    }
}
