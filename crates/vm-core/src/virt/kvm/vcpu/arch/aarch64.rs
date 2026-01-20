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
    fn run(&mut self, _device: &mut IoAddressSpace) -> anyhow::Result<()> {
        todo!()
    }
}
