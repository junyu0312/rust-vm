use gdbstub_arch::aarch64::reg::AArch64CoreRegs;

use crate::arch::aarch64::vcpu::reg::{cnthctl_el2::CnthctlEl2, sctlr_el1::SctlrEl1};

pub struct AArch64CoreRegisters {
    pub general_purpose: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
    pub fp: [u128; 32],
    pub fpcr: u64,
    pub fpsr: u64,
}

pub struct AArch64SysRegisters {
    pub mpidr_el1: u64,
    pub sctlr_el1: u64,
    pub cnthctl_el2: u64,
}

pub struct AArch64Registers {
    pub core: AArch64CoreRegisters,
    pub sys: AArch64SysRegisters,
}

impl AArch64Registers {
    pub fn boot_registers(vcpu_id: usize, x0: u64, pc: u64, regs: AArch64Registers) -> Self {
        let mut new_general_purpose = regs.core.general_purpose;
        new_general_purpose[0] = x0;
        new_general_purpose[1] = 0;
        new_general_purpose[2] = 0;
        new_general_purpose[3] = 0;

        let mut new_pstate = regs.core.pstate;
        new_pstate |= 0x03C0; // DAIF
        new_pstate &= !0xf; // Clear low 4 bits
        new_pstate |= 0x0005; // El1h

        let mut sctlr_el1 = SctlrEl1::from_bits_retain(regs.sys.sctlr_el1);
        sctlr_el1.remove(SctlrEl1::M); // Disable MMU
        sctlr_el1.remove(SctlrEl1::I); // Disable I-cache

        let mut cnthctl_el2 = CnthctlEl2::from_bits_retain(regs.sys.cnthctl_el2);
        cnthctl_el2.insert(CnthctlEl2::EL1PCTEN);

        AArch64Registers {
            core: AArch64CoreRegisters {
                general_purpose: new_general_purpose,
                pc,
                pstate: new_pstate,
                ..regs.core
            },
            sys: AArch64SysRegisters {
                mpidr_el1: vcpu_id as u64,
                sctlr_el1: sctlr_el1.bits(),
                cnthctl_el2: cnthctl_el2.bits(),
            },
        }
    }
}

impl From<AArch64CoreRegs> for AArch64CoreRegisters {
    fn from(regs: AArch64CoreRegs) -> Self {
        AArch64CoreRegisters {
            general_purpose: regs.x,
            sp: regs.sp,
            pc: regs.pc,
            pstate: regs.cpsr as u64,
            fp: regs.v,
            fpcr: regs.fpcr as u64,
            fpsr: regs.fpsr as u64,
        }
    }
}

impl From<AArch64CoreRegisters> for AArch64CoreRegs {
    fn from(regs: AArch64CoreRegisters) -> Self {
        AArch64CoreRegs {
            x: regs.general_purpose,
            sp: regs.sp,
            pc: regs.pc,
            cpsr: regs.pstate as u32,
            v: regs.fp,
            fpcr: regs.fpcr as u32,
            fpsr: regs.fpsr as u32,
        }
    }
}
