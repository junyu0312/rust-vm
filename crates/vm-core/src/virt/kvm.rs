use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::Arc;

use kvm_ioctls::*;

use crate::arch::Arch;
use crate::error::Error;
use crate::virt::HypervisorVm;
use crate::virt::Virt;
use crate::virt::kvm::vcpu::KvmVcpu;

mod irq_chip;
mod vcpu;

pub struct KvmVirt<A: Arch> {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
    _mark: PhantomData<A>,
}

impl<A> Virt for KvmVirt<A>
where
    A: Arch,
{
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, Error> {
        todo!()
    }
}
