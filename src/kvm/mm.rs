use crate::kvm::ioctl::kvm_set_user_memory_region;
use crate::kvm::vm::KvmVm;
use crate::mm::MemoryRegion;

#[repr(C)]
pub struct KvmUserspaceMemoryRegion {
    pub slot: u32,
    pub flags: u32,
    pub guest_phys_addr: u64,
    /// Bytes
    pub memory_size: u64,
    pub userspace_addr: u64,
}

impl KvmVm {
    pub fn init_mm(&mut self, memory_gb: usize) -> anyhow::Result<()> {
        let memory_region = MemoryRegion::new(memory_gb)?;

        let region = KvmUserspaceMemoryRegion {
            slot: 0,
            flags: 0,
            guest_phys_addr: 0x0,
            memory_size: (memory_gb as u64) << 30,
            userspace_addr: memory_region.as_u64(),
        };

        kvm_set_user_memory_region(self.vm_fd, &region)?;

        Ok(())
    }
}
