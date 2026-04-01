use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::Arc;

use kvm_ioctls::*;

use crate::arch::Arch;
use crate::arch::irq::InterruptController;
use crate::arch::vcpu::Vcpu;
use crate::error::Error;
use crate::error::Result;
use crate::virt::DeviceVmExitHandler;
use crate::virt::Virt;
use crate::virt::kvm::irq_chip::KvmIRQ;
use crate::virt::kvm::vcpu::KvmVcpu;

mod irq_chip;
mod vcpu;

#[allow(unused)]
pub struct KvmVirt<A: Arch> {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
    _mark: PhantomData<A>,
}

impl<A> Virt for KvmVirt<A>
where
    A: Arch,
    KvmVcpu: Vcpu<A>,
{
    type Arch = A;

    fn new(_cpu_number: usize) -> Result<Self> {
        let kvm = Kvm::new()
            .map_err(|_| Error::FailedInitialize("kvm: Failed to open /dev/kvm".to_string()))?;

        let vm_fd = Arc::new(
            kvm.create_vm()
                .map_err(|_| Error::FailedInitialize("kvm: Failed to create_vm".to_string()))?,
        );

        Ok(KvmVirt {
            kvm,
            vm_fd,
            vcpus: OnceCell::new(),
            _mark: PhantomData,
        })
    }

    fn create_irq_chip(&mut self) -> Result<Arc<dyn InterruptController>> {
        Ok(Arc::new(KvmIRQ::new(self.vm_fd.clone())?))
    }

    fn set_user_memory_region(
        &mut self,
        _userspace_addr: u64,
        _guest_phys_addr: u64,
        _memory_size: usize,
        _flags: super::SetUserMemoryRegionFlags,
    ) -> Result<()> {
        todo!()
    }

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout {
        todo!()
    }

    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout {
        todo!()
    }

    fn get_vcpu_number(&self) -> usize {
        todo!()
    }

    fn run(&mut self, device_vm_exit_handler: &dyn DeviceVmExitHandler) -> Result<()> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| Error::Internal("vcpus is not init".to_string()))?;

        assert_eq!(vcpus.len(), 1);

        vcpus.get_mut(0).unwrap().run(device_vm_exit_handler)?;

        Ok(())
    }
}
