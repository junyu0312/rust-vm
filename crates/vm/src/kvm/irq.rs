use std::sync::Arc;

use kvm_ioctls::VmFd;
use tracing::error;
use vm_device::device::irq::InterruptController;

use crate::kvm::vm::KvmVm;

pub struct KvmIRQ {
    vm_fd: Arc<VmFd>,
}

impl KvmIRQ {
    pub fn new(vm: &KvmVm) -> anyhow::Result<Self> {
        vm.vm_fd.create_irq_chip()?;

        Ok(KvmIRQ {
            vm_fd: vm.vm_fd.clone(),
        })
    }
}

impl KvmIRQ {
    pub fn set_irq_line(&self, irq: u32, active: bool) -> anyhow::Result<()> {
        self.vm_fd.set_irq_line(irq, active)?;

        Ok(())
    }
}

impl InterruptController for KvmIRQ {
    fn trigger_irq(&self, irq: u32, active: bool) {
        if let Err(err) = self.set_irq_line(irq, active) {
            error!(?err, "Failed to set_irq_line")
        }
    }
}
