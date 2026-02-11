use std::sync::Arc;

use kvm_ioctls::VmFd;
use tracing::error;

use crate::irq::InterruptController;

mod arch;

pub struct KvmIRQ {
    vm_fd: Arc<VmFd>,
}

impl KvmIRQ {
    pub fn new(vm_fd: Arc<VmFd>) -> anyhow::Result<Self> {
        vm_fd.create_irq_chip()?;

        Ok(KvmIRQ { vm_fd })
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
        // debug!(irq, active);
        if let Err(err) = self.set_irq_line(irq, active) {
            error!(?err, "Failed to set_irq_line")
        }
    }

    fn send_msi(&self, _intid: u32) {
        todo!()
    }
}
