use kvm_bindings::kvm_regs;
use kvm_bindings::kvm_sregs;
use kvm_ioctls::Kvm;

use crate::device::pio::IoAddressSpace;
use crate::vcpu::Vcpu;
use crate::virt::kvm::vcpu::KvmVcpu;

impl KvmVcpu {
    pub fn init_arch_vcpu(&self, _kvm: &Kvm) -> anyhow::Result<()> {
        todo!()
    }
}

impl Vcpu for KvmVcpu {
    fn get_regs(&self) -> anyhow::Result<kvm_regs> {
        todo!()
    }

    fn set_regs(&mut self, _regs: &kvm_regs) -> anyhow::Result<()> {
        todo!()
    }

    fn get_sregs(&self) -> anyhow::Result<kvm_sregs> {
        todo!()
    }

    fn set_sregs(&self, _sregs: &kvm_sregs) -> anyhow::Result<()> {
        todo!()
    }

    fn run(&mut self, _device: &mut IoAddressSpace) -> anyhow::Result<()> {
        todo!()
    }
}
