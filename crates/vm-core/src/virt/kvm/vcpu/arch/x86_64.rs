use kvm_bindings::*;
use kvm_ioctls::Kvm;

use crate::arch::x86_64::vcpu::X86_64Vcpu;
use crate::arch::x86_64::vm_exit::VmExitReason;
use crate::vcpu::error::VcpuError;
use crate::virt::kvm::vcpu::KvmVcpu;
use crate::virt::vcpu::Vcpu;

impl KvmVcpu {
    pub fn set_cpuid2(&self, cpuid: &CpuId) -> Result<(), VcpuError> {
        self.vcpu_fd.set_cpuid2(cpuid)?;

        Ok(())
    }

    pub fn init_arch_vcpu(&self, kvm: &Kvm) -> Result<(), VcpuError> {
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

    pub fn set_guest_debug(&self, ctl: &kvm_guest_debug) -> Result<(), VcpuError> {
        self.vcpu_fd.set_guest_debug(ctl)?;

        Ok(())
    }
}

impl X86_64Vcpu for KvmVcpu {
    fn get_regs(&self) -> Result<kvm_regs, VcpuError> {
        let regs = self.vcpu_fd.get_regs()?;

        Ok(regs)
    }

    fn set_regs(&mut self, regs: &kvm_regs) -> Result<(), VcpuError> {
        self.vcpu_fd.set_regs(regs)?;

        Ok(())
    }

    fn get_sregs(&self) -> Result<kvm_bindings::kvm_sregs, VcpuError> {
        let sregs = self.vcpu_fd.get_sregs()?;

        Ok(sregs)
    }

    fn set_sregs(&self, sregs: &kvm_bindings::kvm_sregs) -> Result<(), VcpuError> {
        self.vcpu_fd.set_sregs(sregs)?;

        Ok(())
    }
}

impl Vcpu for KvmVcpu {
    fn post_init_within_thread(&mut self) -> Result<(), VcpuError> {
        todo!()
    }

    fn run(&mut self) -> Result<VmExitReason, VcpuError> {
        todo!()
    }
}
