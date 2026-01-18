use kvm_bindings::kvm_regs;
use kvm_bindings::kvm_sregs;

use crate::device::pio::IoAddressSpace;

pub trait Vcpu {
    fn get_regs(&self) -> anyhow::Result<kvm_regs>;

    fn set_regs(&mut self, regs: &kvm_regs) -> anyhow::Result<()>;

    fn get_sregs(&self) -> anyhow::Result<kvm_sregs>;

    fn set_sregs(&self, sregs: &kvm_sregs) -> anyhow::Result<()>;

    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()>;
}
