use kvm_bindings::CpuId;
use kvm_bindings::kvm_sregs;

use crate::kvm::vcpu::KvmVcpu;

impl KvmVcpu {
    pub fn get_sregs(&self) -> anyhow::Result<kvm_sregs> {
        let sregs = self.vcpu_fd.get_sregs()?;

        Ok(sregs)
    }

    pub fn set_sregs(&self, sregs: &kvm_sregs) -> anyhow::Result<()> {
        self.vcpu_fd.set_sregs(sregs)?;

        Ok(())
    }

    pub fn set_cpuid2(&self, cpuid: &CpuId) -> anyhow::Result<()> {
        self.vcpu_fd.set_cpuid2(cpuid)?;

        Ok(())
    }
    /*
        pub fn init_arch_vcpu(&self, kvm: &Kvm) -> anyhow::Result<()> {
            let mut sregs = self.get_sregs()?;

            let kernel_load = KERNEL_LOAD_ADDR;

            let segment_base = kernel_load >> 4;

            {
                let segment_base = segment_base + 0x20;
                sregs.cs.base = (segment_base as u64) << 4;
                sregs.cs.selector = segment_base as u16;
                sregs.cs.limit = 0xFFFF;
                sregs.cs.present = 1;
                sregs.cs.type_ = 3;
                sregs.cs.s = 1;
            }

            {
                for seg in [
                    &mut sregs.ss,
                    &mut sregs.ds,
                    &mut sregs.es,
                    &mut sregs.fs,
                    &mut sregs.gs,
                ] {
                    seg.base = (segment_base as u64) << 4;
                    seg.selector = segment_base as u16;
                    seg.limit = 0xFFFF;
                    seg.present = 1;
                    seg.type_ = 3;
                    seg.s = 1;
                }
            }

            sregs.cs.limit = 0xFFFF;
            sregs.ds.limit = 0xFFFF;
            sregs.es.limit = 0xFFFF;
            sregs.fs.limit = 0xFFFF;
            sregs.gs.limit = 0xFFFF;
            sregs.ss.limit = 0xFFFF;
            sregs.cr0 = 0;
            sregs.efer = 0;
            self.set_sregs(&sregs)?;

            let mut regs = self.get_regs()?;
            regs.rip = 0x0000;
            regs.rflags = 2;
            regs.rsp = 0x9800;
            self.set_regs(&regs)?;

            self.set_guest_debug(&kvm_guest_debug {
                control: KVM_GUESTDBG_ENABLE | KVM_GUESTDBG_SINGLESTEP,
                pad: 0,
                arch: kvm_guest_debug_arch { debugreg: [0; 8] },
            })?;

            let cpuid2 = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)?;
            self.set_cpuid2(&cpuid2)?;

            Ok(())
        }
    */
}
