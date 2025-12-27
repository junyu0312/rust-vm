use anyhow::anyhow;
use kvm_ioctls::VcpuExit;
use kvm_ioctls::VcpuFd;

use crate::kvm::vm::KvmVm;

#[derive(Debug)]
pub struct KvmVcpu {
    #[allow(dead_code)]
    vcpu_id: u64,
    vcpu_fd: VcpuFd,
}

impl KvmVm {
    pub fn create_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()> {
        let mut vcpus = Vec::with_capacity(num_vcpus);

        for vcpu_id in 0..num_vcpus {
            let vcpu_id = vcpu_id as u64;
            let vcpu_fd = self.vm_fd.create_vcpu(vcpu_id)?;

            vcpus.push(KvmVcpu { vcpu_id, vcpu_fd });
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

        match vcpu.vcpu_fd.run()? {
            VcpuExit::IoOut(_, _) => todo!(),
            VcpuExit::IoIn(_, _) => todo!(),
            VcpuExit::MmioRead(_, _) => todo!(),
            VcpuExit::MmioWrite(_, _) => todo!(),
            VcpuExit::Unknown => todo!(),
            VcpuExit::Exception => todo!(),
            VcpuExit::Hypercall(_) => todo!(),
            VcpuExit::Debug(_) => todo!(),
            VcpuExit::Hlt => todo!(),
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

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.run_vcpu(0)?;

        Ok(())
    }
}
