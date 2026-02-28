use kvm_bindings::kvm_regs;
use kvm_bindings::kvm_sregs;

use crate::arch::vcpu::Vcpu;
use crate::arch::x86_64::X86_64;
use crate::error::Result;

pub trait X86Vcpu: Vcpu<X86_64> {
    fn get_regs(&self) -> Result<kvm_regs>;

    fn set_regs(&mut self, regs: &kvm_regs) -> Result<()>;

    fn get_sregs(&self) -> Result<kvm_sregs>;

    fn set_sregs(&self, sregs: &kvm_sregs) -> Result<()>;
}
