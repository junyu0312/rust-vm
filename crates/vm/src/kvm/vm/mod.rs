use std::cell::OnceCell;
use std::sync::Arc;

use anyhow::anyhow;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::Kvm;
use kvm_ioctls::VmFd;
use vm_core::mm::manager::MemoryRegions;
use vm_core::mm::region::MemoryRegion;
use vm_device::bus::io_address_space::IoAddressSpace;

use crate::kvm::vcpu::KvmVcpu;

pub struct KvmVm {
    pub kvm: Kvm,
    pub vm_fd: Arc<VmFd>,
    pub vcpus: OnceCell<Vec<KvmVcpu>>,
    pub memory_regions: OnceCell<MemoryRegions>,
    pub ram_size: usize,
    pub io_address_space: OnceCell<IoAddressSpace>,
}

impl KvmVm {
    pub fn new(kvm: Kvm) -> anyhow::Result<Self> {
        let vm_fd = kvm.create_vm()?;
        Ok(KvmVm {
            kvm,
            vm_fd: Arc::new(vm_fd),
            vcpus: Default::default(),
            memory_regions: Default::default(),
            ram_size: Default::default(),
            io_address_space: Default::default(),
        })
    }

    pub fn init_mm(&mut self, len: usize) -> anyhow::Result<()> {
        let memory_region = MemoryRegion::new(0, len)?;

        unsafe {
            self.vm_fd
                .set_user_memory_region(kvm_userspace_memory_region {
                    slot: 0,
                    flags: 0,
                    guest_phys_addr: 0x0,
                    memory_size: len as u64,
                    userspace_addr: memory_region.as_u64(),
                })?;
        }

        let mut memory_regions = MemoryRegions::default();
        memory_regions
            .try_insert(memory_region)
            .map_err(|_| anyhow!("Failed to insert memory_region"))?;

        self.memory_regions
            .set(memory_regions)
            .map_err(|_| anyhow!("memory regions are already set"))?;
        self.ram_size = len;

        Ok(())
    }
}
