use std::sync::Arc;

use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::VmFd;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::irq::InterruptController;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::kvm::irq_chip::KvmIrqChip;
use crate::virtualization::kvm::vcpu::KvmVcpu;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vm::HypervisorVm;
use crate::virtualization::vm::SetUserMemoryRegionFlags;
use crate::virtualization::vm::error::VmError;

pub struct KvmVm {
    vm_fd: VmFd,
}

impl KvmVm {
    pub fn new(vm_fd: VmFd) -> Self {
        KvmVm { vm_fd }
    }
}

impl HypervisorVm for KvmVm {
    fn create_vcpu(
        &self,
        vcpu_id: usize,
        _mm: Arc<MemoryAddressSpace>,
        _vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Box<dyn HypervisorVcpu>, VmError> {
        let vcpu = KvmVcpu::new(&self.vm_fd, vcpu_id as u64)
            .map_err(|err| VmError::CreateVcpuError(Box::new(err)))?;

        Ok(Box::new(vcpu))
    }

    fn create_irq_chip(&self) -> Result<Box<dyn InterruptController>, VmError> {
        self.vm_fd.create_irq_chip()?;

        let irq_chip = KvmIrqChip {};

        Ok(Box::new(irq_chip))
    }

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        _flags: SetUserMemoryRegionFlags,
    ) -> Result<(), VmError> {
        (unsafe {
            self.vm_fd
                .set_user_memory_region(kvm_userspace_memory_region {
                    slot: 0,
                    flags: 0,
                    guest_phys_addr,
                    memory_size: memory_size as u64,
                    userspace_addr,
                })
        })?;

        Ok(())
    }
}
