use kvm_bindings::*;
use kvm_ioctls::Kvm;
use kvm_ioctls::VcpuExit;
use tracing::error;

use crate::device::pio::IoAddressSpace;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::x86_64::X86Vcpu;
use crate::virt::kvm::vcpu::KvmVcpu;

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

impl X86Vcpu for KvmVcpu {
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

impl Vcpu for KvmVcpu {
    fn run(&mut self, pio: &mut IoAddressSpace) -> anyhow::Result<()> {
        loop {
            // trace!("al: {}", vcpu.get_regs()?.rax & 0xFF);

            let r = self.vcpu_fd.run();

            // trace!("{:?}", r);
            match r? {
                VcpuExit::IoOut(port, data) => pio.io_out(port, data)?,
                VcpuExit::IoIn(port, data) => pio.io_in(port, data)?,
                VcpuExit::MmioRead(_, _) => {
                    // Ignore
                }
                VcpuExit::MmioWrite(_, _) => {
                    // Ignore
                }
                VcpuExit::Unknown => todo!(),
                VcpuExit::Exception => todo!(),
                VcpuExit::Hypercall(_) => todo!(),
                VcpuExit::Debug(_) => {}
                VcpuExit::Hlt => {
                    // warn!("hlt");
                    // todo!()
                }
                VcpuExit::IrqWindowOpen => todo!(),
                VcpuExit::Shutdown => todo!(),
                VcpuExit::FailEntry(_, _) => todo!(),
                VcpuExit::Intr => todo!(),
                VcpuExit::SetTpr => todo!(),
                VcpuExit::TprAccess => todo!(),
                VcpuExit::S390Sieic => todo!(),
                VcpuExit::S390Reset => todo!(),
                VcpuExit::Dcr => todo!(),
                VcpuExit::Nmi => todo!(),
                VcpuExit::InternalError => {
                    let kvm_run = self.vcpu_fd.get_kvm_run();
                    unsafe {
                        error!(?kvm_run.__bindgen_anon_1.internal, "InternalError");
                    }
                    panic!();
                }
                VcpuExit::Osi => todo!(),
                VcpuExit::PaprHcall => todo!(),
                VcpuExit::S390Ucontrol => todo!(),
                VcpuExit::Watchdog => todo!(),
                VcpuExit::S390Tsch => todo!(),
                VcpuExit::Epr => todo!(),
                VcpuExit::SystemEvent(_, _) => todo!(),
                VcpuExit::S390Stsi => todo!(),
                VcpuExit::IoapicEoi(_) => todo!(),
                VcpuExit::Hyperv => todo!(),
                VcpuExit::X86Rdmsr(_) => todo!(),
                VcpuExit::X86Wrmsr(_) => todo!(),
                VcpuExit::MemoryFault { .. } => todo!(),
                VcpuExit::Unsupported(_) => todo!(),
            }
        }
    }
}
