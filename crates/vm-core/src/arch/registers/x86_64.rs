use gdbstub_arch::x86::reg::X86_64CoreRegs;

pub struct X86_64CoreRegisters;

pub struct X86_64Registers;

impl From<X86_64CoreRegs> for X86_64CoreRegisters {
    fn from(_regs: X86_64CoreRegs) -> X86_64CoreRegisters {
        todo!()
    }
}

impl From<X86_64CoreRegisters> for X86_64CoreRegs {
    fn from(_regs: X86_64CoreRegisters) -> X86_64CoreRegs {
        todo!()
    }
}
