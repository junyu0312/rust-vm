use kvm_bindings::kvm_guest_debug;
use kvm_bindings::kvm_regs;
use kvm_ioctls::VcpuFd;
use vm_core::vcpu::Vcpu;

#[derive(Debug)]
pub struct KvmVcpu {
    pub vcpu_id: u64,
    pub vcpu_fd: VcpuFd,
}

impl KvmVcpu {
    pub fn set_regs(&self, regs: &kvm_regs) -> anyhow::Result<()> {
        self.vcpu_fd.set_regs(regs)?;

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
