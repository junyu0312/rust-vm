use std::cell::OnceCell;
use std::sync::Arc;

use kvm_ioctls::*;

use crate::hypervisor::Hypervisor;
use crate::hypervisor::HypervisorError;
use crate::hypervisor::HypervisorVm;
use crate::hypervisor::kvm::vcpu::KvmVcpu;

mod irq_chip;
mod vcpu;

pub struct KvmVirt {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
}

impl Hypervisor for KvmVirt {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError> {
        todo!()
    }
}
