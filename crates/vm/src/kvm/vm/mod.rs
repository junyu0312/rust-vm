use std::cell::OnceCell;

use kvm_ioctls::Kvm;
use kvm_ioctls::VmFd;
use vm_device::bus::io_address_space::IoAddressSpace;

use crate::kvm::vcpu::KvmVcpu;
use crate::mm::manager::MemoryRegions;

pub struct KvmVm {
    pub kvm: Kvm,
    pub vm_fd: VmFd,
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
            vm_fd,
            vcpus: Default::default(),
            memory_regions: Default::default(),
            ram_size: Default::default(),
            io_address_space: Default::default(),
        })
    }
}
