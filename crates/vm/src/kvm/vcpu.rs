use anyhow::anyhow;
use kvm_bindings::KVM_GUESTDBG_ENABLE;
use kvm_bindings::KVM_GUESTDBG_SINGLESTEP;
use kvm_bindings::kvm_guest_debug;
use kvm_bindings::kvm_guest_debug_arch;
use kvm_bindings::kvm_regs;
use kvm_bindings::kvm_sregs;
use kvm_ioctls::VcpuExit;
use kvm_ioctls::VcpuFd;
use tracing::debug;
use tracing::trace;

use crate::kvm::loader::KERNEL_LOAD_ADDR;
use crate::kvm::vm::KvmVm;

#[derive(Debug)]
pub struct KvmVcpu {
    #[allow(dead_code)]
    vcpu_id: u64,
    vcpu_fd: VcpuFd,
}

impl KvmVcpu {
    fn get_regs(&self) -> anyhow::Result<kvm_regs> {
        let regs = self.vcpu_fd.get_regs()?;

        Ok(regs)
    }

    fn set_regs(&self, regs: &kvm_regs) -> anyhow::Result<()> {
        self.vcpu_fd.set_regs(regs)?;

        Ok(())
    }

    fn get_sregs(&self) -> anyhow::Result<kvm_sregs> {
        let sregs = self.vcpu_fd.get_sregs()?;

        Ok(sregs)
    }

    fn set_sregs(&self, sregs: &kvm_sregs) -> anyhow::Result<()> {
        self.vcpu_fd.set_sregs(sregs)?;

        Ok(())
    }

    fn set_guest_debug(&self, ctl: &kvm_guest_debug) -> anyhow::Result<()> {
        self.vcpu_fd.set_guest_debug(ctl)?;

        Ok(())
    }
}

impl KvmVm {
    fn create_vcpu(&self, id: u64) -> anyhow::Result<KvmVcpu> {
        let vcpu_fd = self.vm_fd.create_vcpu(id)?;

        Ok(KvmVcpu {
            vcpu_id: id,
            vcpu_fd,
        })
    }

    pub fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()> {
        let mut vcpus = Vec::with_capacity(num_vcpus);

        for vcpu_id in 0..num_vcpus {
            let vcpu_id = vcpu_id as u64;
            let vcpu_fd = self.create_vcpu(vcpu_id)?;

            let mut sregs = vcpu_fd.get_sregs()?;

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
            vcpu_fd.set_sregs(&sregs)?;

            let mut regs = vcpu_fd.get_regs()?;
            regs.rip = 0x0000;
            regs.rflags = 2;
            regs.rsp = 0x9800;
            vcpu_fd.set_regs(&regs)?;

            vcpu_fd.set_guest_debug(&kvm_guest_debug {
                control: KVM_GUESTDBG_ENABLE | KVM_GUESTDBG_SINGLESTEP,
                pad: 0,
                arch: kvm_guest_debug_arch { debugreg: [0; 8] },
            })?;

            vcpus.push(vcpu_fd);
        }

        self.vcpus
            .set(vcpus)
            .map_err(|_| anyhow!("vcpus are already set"))?;

        Ok(())
    }

    fn run_vcpu(&mut self, i: usize) -> anyhow::Result<()> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus are not created"))?;

        let vcpu = &mut vcpus[i];

        debug!(
            "Starting vCPU {}, regs: {:?}, sregs: {:?}",
            i,
            vcpu.get_regs(),
            vcpu.get_sregs()
        );

        loop {
            trace!("al: {}", vcpu.get_regs()?.rax & 0xFF);

            let r = vcpu.vcpu_fd.run();

            trace!("{:?}", r);
            match r? {
                VcpuExit::IoOut(port, data) => {
                    debug!("IoOut: port: {:#X}, data: {:?}", port, data);
                    todo!()
                }
                VcpuExit::IoIn(port, data) => {
                    debug!("IoIn: port: {:#X}, data: {:?}", port, data);
                    todo!()
                }
                VcpuExit::MmioRead(_, _) => todo!(),
                VcpuExit::MmioWrite(_, _) => todo!(),
                VcpuExit::Unknown => todo!(),
                VcpuExit::Exception => todo!(),
                VcpuExit::Hypercall(_) => todo!(),
                VcpuExit::Debug(_) => {}
                VcpuExit::Hlt => {
                    todo!()
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
                VcpuExit::InternalError => todo!(),
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

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.run_vcpu(0)?;

        Ok(())
    }
}
