use anyhow::anyhow;
use kvm_bindings::kvm_guest_debug;
use kvm_bindings::kvm_regs;
use kvm_ioctls::VcpuExit;
use kvm_ioctls::VcpuFd;
// use tracing::trace;

use crate::kvm::vm::KvmVm;

#[derive(Debug)]
pub struct KvmVcpu {
    #[allow(dead_code)]
    pub vcpu_id: u64,
    pub vcpu_fd: VcpuFd,
}

impl KvmVcpu {
    pub fn get_regs(&self) -> anyhow::Result<kvm_regs> {
        let regs = self.vcpu_fd.get_regs()?;

        Ok(regs)
    }

    pub fn set_regs(&self, regs: &kvm_regs) -> anyhow::Result<()> {
        self.vcpu_fd.set_regs(regs)?;

        Ok(())
    }

    pub fn set_guest_debug(&self, ctl: &kvm_guest_debug) -> anyhow::Result<()> {
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

            vcpu_fd.init_arch_vcpu(&self.kvm)?;

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

        loop {
            // trace!("al: {}", vcpu.get_regs()?.rax & 0xFF);

            let r = vcpu.vcpu_fd.run();

            // trace!("{:?}", r);
            match r? {
                VcpuExit::IoOut(port, data) => {
                    self.pio_bus
                        .get_mut()
                        .ok_or_else(|| anyhow!("pio_bus is not initialized"))?
                        .io_out(port, data)?;
                }
                VcpuExit::IoIn(port, data) => {
                    self.pio_bus
                        .get_mut()
                        .ok_or_else(|| anyhow!("pio_bus is not initialized"))?
                        .io_in(port, data)?;
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
