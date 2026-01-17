use kvm_bindings::CpuId;
use kvm_bindings::KVM_GUESTDBG_ENABLE;
use kvm_bindings::KVM_GUESTDBG_SINGLESTEP;
use kvm_bindings::KVM_MAX_CPUID_ENTRIES;
use kvm_bindings::kvm_guest_debug;
use kvm_bindings::kvm_guest_debug_arch;
use kvm_bindings::kvm_sregs;
use kvm_ioctls::Kvm;

use crate::kvm::vcpu::KvmVcpu;

impl KvmVcpu {
    pub fn set_sregs(&self, sregs: &kvm_sregs) -> anyhow::Result<()> {
        self.vcpu_fd.set_sregs(sregs)?;

        Ok(())
    }

    pub fn set_cpuid2(&self, cpuid: &CpuId) -> anyhow::Result<()> {
        self.vcpu_fd.set_cpuid2(cpuid)?;

        Ok(())
    }

    pub fn init_arch_vcpu(&self, kvm: &Kvm) -> anyhow::Result<()> {
        let mut cpuid = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)?;

        let entries = cpuid.as_mut_slice();
        for entry in entries.iter_mut() {
            if entry.function == 0x1 && entry.index == 0 {
                entry.ecx |= 1 << 31;
            }
        }

        self.set_cpuid2(&cpuid)?;

        self.set_guest_debug(&kvm_guest_debug {
            control: KVM_GUESTDBG_ENABLE | KVM_GUESTDBG_SINGLESTEP,
            pad: 0,
            arch: kvm_guest_debug_arch { debugreg: [0; 8] },
        })?;

        Ok(())
    }
}
