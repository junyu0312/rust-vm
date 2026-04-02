use kvm_bindings::kvm_regs;
use kvm_bindings::kvm_sregs;

use crate::cpu::error::VcpuError;

pub trait X86_64Vcpu {
    fn get_regs(&self) -> Result<kvm_regs, VcpuError>;

    fn set_regs(&mut self, regs: &kvm_regs) -> Result<(), VcpuError>;

    fn get_sregs(&self) -> Result<kvm_sregs, VcpuError>;

    fn set_sregs(&self, sregs: &kvm_sregs) -> Result<(), VcpuError>;
}
