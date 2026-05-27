use gdbstub_arch::aarch64::reg::AArch64CoreRegs;
use serde::Deserialize;
use serde::Serialize;
use vm_aarch64::register::cnthctl_el2::CnthctlEl2;
use vm_aarch64::register::sctlr_el1::SctlrEl1;

#[derive(Serialize, Deserialize)]
pub struct AArch64CoreRegisters {
    pub general_purpose: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
    pub fp: [u128; 32],
    pub fpcr: u64,
    pub fpsr: u64,
}

#[derive(Serialize, Deserialize)]
pub struct AArch64SysRegisters {
    pub dbgbvr0_el1: u64,
    pub dbgbcr0_el1: u64,
    pub dbgwvr0_el1: u64,
    pub dbgwcr0_el1: u64,
    pub dbgbvr1_el1: u64,
    pub dbgbcr1_el1: u64,
    pub dbgwvr1_el1: u64,
    pub dbgwcr1_el1: u64,
    pub mdccint_el1: u64,
    pub mdscr_el1: u64,
    pub dbgbvr2_el1: u64,
    pub dbgbcr2_el1: u64,
    pub dbgwvr2_el1: u64,
    pub dbgwcr2_el1: u64,
    pub dbgbvr3_el1: u64,
    pub dbgbcr3_el1: u64,
    pub dbgwvr3_el1: u64,
    pub dbgwcr3_el1: u64,
    pub dbgbvr4_el1: u64,
    pub dbgbcr4_el1: u64,
    pub dbgwvr4_el1: u64,
    pub dbgwcr4_el1: u64,
    pub dbgbvr5_el1: u64,
    pub dbgbcr5_el1: u64,
    pub dbgwvr5_el1: u64,
    pub dbgwcr5_el1: u64,
    pub dbgbvr6_el1: u64,
    pub dbgbcr6_el1: u64,
    pub dbgwvr6_el1: u64,
    pub dbgwcr6_el1: u64,
    pub dbgbvr7_el1: u64,
    pub dbgbcr7_el1: u64,
    pub dbgwvr7_el1: u64,
    pub dbgwcr7_el1: u64,
    pub dbgbvr8_el1: u64,
    pub dbgbcr8_el1: u64,
    pub dbgwvr8_el1: u64,
    pub dbgwcr8_el1: u64,
    pub dbgbvr9_el1: u64,
    pub dbgbcr9_el1: u64,
    pub dbgwvr9_el1: u64,
    pub dbgwcr9_el1: u64,
    pub dbgbvr10_el1: u64,
    pub dbgbcr10_el1: u64,
    pub dbgwvr10_el1: u64,
    pub dbgwcr10_el1: u64,
    pub dbgbvr11_el1: u64,
    pub dbgbcr11_el1: u64,
    pub dbgwvr11_el1: u64,
    pub dbgwcr11_el1: u64,
    pub dbgbvr12_el1: u64,
    pub dbgbcr12_el1: u64,
    pub dbgwvr12_el1: u64,
    pub dbgwcr12_el1: u64,
    pub dbgbvr13_el1: u64,
    pub dbgbcr13_el1: u64,
    pub dbgwvr13_el1: u64,
    pub dbgwcr13_el1: u64,
    pub dbgbvr14_el1: u64,
    pub dbgbcr14_el1: u64,
    pub dbgwvr14_el1: u64,
    pub dbgwcr14_el1: u64,
    pub dbgbvr15_el1: u64,
    pub dbgbcr15_el1: u64,
    pub dbgwvr15_el1: u64,
    pub dbgwcr15_el1: u64,
    pub midr_el1: u64,
    pub mpidr_el1: u64,
    pub id_aa64pfr0_el1: u64,
    pub id_aa64pfr1_el1: u64,
    pub id_aa64zfr0_el1: u64,
    pub id_aa64smfr0_el1: u64,
    pub id_aa64dfr0_el1: u64,
    pub id_aa64dfr1_el1: u64,
    pub id_aa64isar0_el1: u64,
    pub id_aa64isar1_el1: u64,
    pub id_aa64mmfr0_el1: u64,
    pub id_aa64mmfr1_el1: u64,
    pub id_aa64mmfr2_el1: u64,
    pub sctlr_el1: u64,
    pub cpacr_el1: u64,
    pub actlr_el1: u64,
    pub smpri_el1: u64,
    pub smcr_el1: u64,
    pub ttbr0_el1: u64,
    pub ttbr1_el1: u64,
    pub tcr_el1: u64,
    pub apiakeylo_el1: u64,
    pub apiakeyhi_el1: u64,
    pub apibkeylo_el1: u64,
    pub apibkeyhi_el1: u64,
    pub apdakeylo_el1: u64,
    pub apdakeyhi_el1: u64,
    pub apdbkeylo_el1: u64,
    pub apdbkeyhi_el1: u64,
    pub apgakeylo_el1: u64,
    pub apgakeyhi_el1: u64,
    pub spsr_el1: u64,
    pub elr_el1: u64,
    pub sp_el0: u64,
    pub afsr0_el1: u64,
    pub afsr1_el1: u64,
    pub esr_el1: u64,
    pub far_el1: u64,
    pub par_el1: u64,
    pub mair_el1: u64,
    pub amair_el1: u64,
    pub vbar_el1: u64,
    pub contextidr_el1: u64,
    pub tpidr_el1: u64,
    pub scxtnum_el1: u64,
    pub cntkctl_el1: u64,
    pub csselr_el1: u64,
    pub tpidr_el0: u64,
    pub tpidrro_el0: u64,
    pub tpidr2_el0: u64,
    pub scxtnum_el0: u64,
    pub cntv_ctl_el0: u64,
    pub cntv_cval_el0: u64,
    pub sp_el1: u64,
    pub cntp_ctl_el0: u64,
    pub cntp_cval_el0: u64,
    pub cntp_tval_el0: u64,
    pub cnthctl_el2: u64,
    pub cnthp_ctl_el2: u64,
    pub cnthp_cval_el2: u64,
    pub cnthp_tval_el2: u64,
    pub cntvoff_el2: u64,
    pub cptr_el2: u64,
    pub elr_el2: u64,
    pub esr_el2: u64,
    pub far_el2: u64,
    pub hcr_el2: u64,
    pub hpfar_el2: u64,
    pub mair_el2: u64,
    // pub mdcr_el2: u64,
    pub sctlr_el2: u64,
    pub spsr_el2: u64,
    pub sp_el2: u64,
    pub tcr_el2: u64,
    pub tpidr_el2: u64,
    pub ttbr0_el2: u64,
    pub ttbr1_el2: u64,
    pub vbar_el2: u64,
    pub vmpidr_el2: u64,
    pub vpidr_el2: u64,
    pub vtcr_el2: u64,
    pub vttbr_el2: u64,
}

#[derive(Serialize, Deserialize)]
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
                ..regs.sys
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
