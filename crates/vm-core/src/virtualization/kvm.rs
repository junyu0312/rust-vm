use std::sync::Arc;

#[cfg(target_arch = "x86_64")]
use kvm_bindings::CpuId;
#[cfg(target_arch = "x86_64")]
use kvm_bindings::KVM_MAX_CPUID_ENTRIES;
#[cfg(target_arch = "x86_64")]
use kvm_bindings::KVM_PIT_SPEAKER_DUMMY;
#[cfg(target_arch = "x86_64")]
use kvm_bindings::kvm_pit_config;
use kvm_ioctls::Kvm;

use crate::virtualization::hypervisor::Hypervisor;
use crate::virtualization::hypervisor::error::HypervisorError;
use crate::virtualization::kvm::vm::KvmVm;
use crate::virtualization::vm::HypervisorVm;

pub mod gsi_routing;

mod arch;
mod irq_chip;
mod vcpu;
mod vm;

pub struct KvmHypervisor {
    kvm: Kvm,
    #[cfg(target_arch = "x86_64")]
    supported_cpuid_patched: CpuId,
}

impl KvmHypervisor {
    pub fn new() -> Result<Self, HypervisorError> {
        let kvm = Kvm::new()?;
        #[cfg(target_arch = "x86_64")]
        let supported_cpuid_patched = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)?;

        Ok(KvmHypervisor {
            kvm,
            #[cfg(target_arch = "x86_64")]
            supported_cpuid_patched,
        })
    }
}

impl Hypervisor for KvmHypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError> {
        let vm_fd = self.kvm.create_vm()?;

        #[cfg(target_arch = "x86_64")]
        vm_fd.create_pit2(kvm_pit_config {
            flags: KVM_PIT_SPEAKER_DUMMY, // avoid vm_exit from port 0x61
            ..Default::default()
        })?;

        Ok(Arc::new(KvmVm::new(
            vm_fd,
            #[cfg(target_arch = "x86_64")]
            self.supported_cpuid_patched.clone(),
        )))
    }
}
