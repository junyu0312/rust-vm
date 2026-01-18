use kvm_bindings::*;
use kvm_ioctls::*;

use crate::vcpu::Vcpu;

#[derive(Debug)]
pub struct KvmVcpu {
    pub vcpu_id: u64,
    pub vcpu_fd: VcpuFd,
}

impl KvmVcpu {
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

    pub fn set_guest_debug(&self, ctl: &kvm_guest_debug) -> anyhow::Result<()> {
        self.vcpu_fd.set_guest_debug(ctl)?;

        Ok(())
    }
}

impl Vcpu for KvmVcpu {
    fn get_regs(&self) -> anyhow::Result<kvm_regs> {
        let regs = self.vcpu_fd.get_regs()?;

        Ok(regs)
    }

    fn set_regs(&mut self, regs: &kvm_regs) -> anyhow::Result<()> {
        self.vcpu_fd.set_regs(regs)?;

        Ok(())
    }

    fn get_sregs(&self) -> anyhow::Result<kvm_bindings::kvm_sregs> {
        let sregs = self.vcpu_fd.get_sregs()?;

        Ok(sregs)
    }

    fn set_sregs(&self, sregs: &kvm_bindings::kvm_sregs) -> anyhow::Result<()> {
        self.vcpu_fd.set_sregs(sregs)?;

        Ok(())
    }
}
