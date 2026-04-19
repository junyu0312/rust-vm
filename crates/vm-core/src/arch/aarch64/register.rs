use crate::arch::aarch64::vcpu::reg::cnthctl_el2::CnthctlEl2;
use crate::arch::aarch64::vcpu::reg::sctlr_el1::SctlrEl1;

pub struct AArch64Registers {
    // Core
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub pc: u64,
    pub pstate: u64,

    // Sys
    pub mpidr_el1: u64,
    pub sctlr_el1: u64,
    pub cnthctl_el2: u64,
}

impl AArch64Registers {
    pub fn boot_registers(
        vcpu_id: usize,
        x0: u64,
        pc: u64,
        mut pstate: u64,
        sctlr_el1: u64,
        cnthctl_el2: u64,
    ) -> Self {
        pstate |= 0x03C0; // DAIF
        pstate &= !0xf; // Clear low 4 bits
        pstate |= 0x0005; // El1h

        let mut sctlr_el1 = SctlrEl1::from_bits_retain(sctlr_el1);
        sctlr_el1.remove(SctlrEl1::M); // Disable MMU
        sctlr_el1.remove(SctlrEl1::I); // Disable I-cache

        let mut cnthctl_el2 = CnthctlEl2::from_bits_retain(cnthctl_el2);
        cnthctl_el2.insert(CnthctlEl2::EL1PCTEN);

        AArch64Registers {
            x0,
            x1: 0,
            x2: 0,
            x3: 0,
            pc,
            pstate,
            mpidr_el1: vcpu_id as u64,
            sctlr_el1: sctlr_el1.bits(),
            cnthctl_el2: cnthctl_el2.bits(),
        }
    }
}
