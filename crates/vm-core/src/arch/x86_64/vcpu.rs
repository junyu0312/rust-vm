use crate::arch::registers::x86_64::X86_64CoreRegisters;
use crate::arch::registers::x86_64::X86_64Registers;
use crate::arch::registers::x86_64::X86_64SRegisters;
use crate::virtualization::vcpu::error::VcpuError;

pub trait X86_64Vcpu {
    fn get_regs(&self) -> Result<X86_64Registers, VcpuError>;

    fn set_regs(&mut self, regs: X86_64Registers) -> Result<(), VcpuError>;

    fn get_core_regs(&self) -> Result<X86_64CoreRegisters, VcpuError>;

    fn set_core_regs(&mut self, regs: X86_64CoreRegisters) -> Result<(), VcpuError>;

    fn get_sregs(&self) -> Result<X86_64SRegisters, VcpuError>;

    fn set_sregs(&self, sregs: X86_64SRegisters) -> Result<(), VcpuError>;
}
