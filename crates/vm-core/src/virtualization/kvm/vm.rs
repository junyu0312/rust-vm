use std::sync::Arc;

#[cfg(target_arch = "x86_64")]
use kvm_bindings::CpuId;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::VmFd;
use vm_mm::manager::MemoryAddressSpace;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::layout::IRQ_ALLOCATION_END;
#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::layout::IRQ_ALLOCATION_START;
use crate::arch::irq::InterruptController;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::layout::IRQ_ALLOCATION_END;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::layout::IRQ_ALLOCATION_START;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::irq_allocator::IrqAllocator;
use crate::virtualization::kvm::irq_chip::KvmIrqChip;
use crate::virtualization::kvm::vcpu::KvmVcpu;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vm::HypervisorVm;
use crate::virtualization::vm::SetUserMemoryRegionFlags;
use crate::virtualization::vm::error::VmError;

pub struct KvmVm {
    vm_fd: Arc<VmFd>,
    #[cfg(target_arch = "x86_64")]
    supported_cpuid_patched: CpuId,
}

impl KvmVm {
    pub fn new(vm_fd: VmFd, #[cfg(target_arch = "x86_64")] supported_cpuid_patched: CpuId) -> Self {
        KvmVm {
            vm_fd: Arc::new(vm_fd),
            #[cfg(target_arch = "x86_64")]
            supported_cpuid_patched,
        }
    }
}

impl HypervisorVm for KvmVm {
    fn create_vcpu(
        &self,
        vcpu_id: u64,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Box<dyn HypervisorVcpu>, VmError> {
        let vcpu = KvmVcpu::new(
            &self.vm_fd,
            vcpu_id,
            #[cfg(target_arch = "x86_64")]
            &self.supported_cpuid_patched,
            vm_exit_handler,
            mm,
        )
        .map_err(|err| VmError::CreateVcpuError(Box::new(err)))?;

        Ok(Box::new(vcpu))
    }

    fn create_irq_chip(&self) -> Result<Box<dyn InterruptController>, VmError> {
        self.vm_fd.create_irq_chip()?;

        let irq_chip = KvmIrqChip {
            vm_fd: self.vm_fd.clone(),
        };

        Ok(Box::new(irq_chip))
    }

    fn create_irq_allocator(&self) -> Result<IrqAllocator, VmError> {
        Ok(IrqAllocator::new(IRQ_ALLOCATION_START, IRQ_ALLOCATION_END))
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
