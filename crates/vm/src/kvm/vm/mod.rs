use std::cell::OnceCell;

use kvm_ioctls::Kvm;
use kvm_ioctls::VmFd;
use vm_device::bus::pio::PioBus;

use crate::kvm::vcpu::KvmVcpu;
use crate::mm::manager::MemoryRegions;

pub struct KvmVm {
    pub kvm: Kvm,
    pub vm_fd: VmFd,
    pub vcpus: OnceCell<Vec<KvmVcpu>>,
    pub memory_regions: OnceCell<MemoryRegions>,
    pub pio_bus: OnceCell<PioBus>,
}

impl KvmVm {
    pub fn new(kvm: Kvm) -> anyhow::Result<Self> {
        let vm_fd = kvm.create_vm()?;
        Ok(KvmVm {
            kvm,
            vm_fd,
            vcpus: Default::default(),
            memory_regions: Default::default(),
            pio_bus: Default::default(),
        })
    }
}
