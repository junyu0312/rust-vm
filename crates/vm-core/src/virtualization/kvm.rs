use std::sync::Arc;

use kvm_ioctls::Kvm;

use crate::virtualization::hypervisor::Hypervisor;
use crate::virtualization::hypervisor::error::HypervisorError;
use crate::virtualization::kvm::vm::KvmVm;
use crate::virtualization::vm::HypervisorVm;

mod irq_chip;
mod vcpu;
mod vm;

pub struct KvmHypervisor {
    kvm: Kvm,
}

impl KvmHypervisor {
    pub fn new() -> Result<Self, HypervisorError> {
        Ok(KvmHypervisor { kvm: Kvm::new()? })
    }
}

impl Hypervisor for KvmHypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError> {
        let vm_fd = self.kvm.create_vm()?;

        Ok(Arc::new(KvmVm::new(vm_fd)))
    }
}
