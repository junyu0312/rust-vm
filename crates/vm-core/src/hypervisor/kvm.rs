use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::Arc;

use kvm_ioctls::*;

use crate::arch::Arch;
use crate::hypervisor::Hypervisor;
use crate::hypervisor::HypervisorError;
use crate::hypervisor::HypervisorVm;
use crate::hypervisor::kvm::vcpu::KvmVcpu;

mod irq_chip;
mod vcpu;

pub struct KvmVirt<A: Arch> {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
    _mark: PhantomData<A>,
}

impl<A> Hypervisor for KvmVirt<A>
where
    A: Arch,
{
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError> {
        todo!()
    }
}
