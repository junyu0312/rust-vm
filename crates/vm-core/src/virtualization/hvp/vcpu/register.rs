use std::collections::BTreeMap;

use applevisor_sys::hv_gic_icc_reg_t;
use applevisor_sys::hv_gic_ich_reg_t;
use applevisor_sys::hv_gic_icv_reg_t;
use applevisor_sys::hv_gic_redistributor_reg_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_simd_fp_reg_t;
use applevisor_sys::hv_sys_reg_t;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::EnumIter;

use crate::arch::aarch64::vcpu::reg::FpRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::virtualization::vcpu::error::VcpuError;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorGicRedistributorReg {
    TYPER = 0x0008,
    PIDR2 = 0xffe8,

    IGROUPR0 = 0x10080,
    ISENABLER0 = 0x10100,
    ICENABLER0 = 0x10180,
    ISPENDR0 = 0x10200,
    ICPENDR0 = 0x10280,
    ISACTIVER0 = 0x10300,
    ICACTIVER0 = 0x10380,

    IPRIORITYR0 = 0x10400,
    IPRIORITYR1 = 0x10404,
    IPRIORITYR2 = 0x10408,
    IPRIORITYR3 = 0x1040c,
    IPRIORITYR4 = 0x10410,
    IPRIORITYR5 = 0x10414,
    IPRIORITYR6 = 0x10418,
    IPRIORITYR7 = 0x1041c,

    ICFGR0 = 0x10c00,
    ICFGR1 = 0x10c04,
}

impl From<hv_gic_redistributor_reg_t> for AppleHypervisorGicRedistributorReg {
    fn from(reg: hv_gic_redistributor_reg_t) -> Self {
        match reg {
            hv_gic_redistributor_reg_t::TYPER => AppleHypervisorGicRedistributorReg::TYPER,
            hv_gic_redistributor_reg_t::PIDR2 => AppleHypervisorGicRedistributorReg::PIDR2,
            hv_gic_redistributor_reg_t::IGROUPR0 => AppleHypervisorGicRedistributorReg::IGROUPR0,
            hv_gic_redistributor_reg_t::ISENABLER0 => {
                AppleHypervisorGicRedistributorReg::ISENABLER0
            }
            hv_gic_redistributor_reg_t::ICENABLER0 => {
                AppleHypervisorGicRedistributorReg::ICENABLER0
            }
            hv_gic_redistributor_reg_t::ISPENDR0 => AppleHypervisorGicRedistributorReg::ISPENDR0,
            hv_gic_redistributor_reg_t::ICPENDR0 => AppleHypervisorGicRedistributorReg::ICPENDR0,
            hv_gic_redistributor_reg_t::ISACTIVER0 => {
                AppleHypervisorGicRedistributorReg::ISACTIVER0
            }
            hv_gic_redistributor_reg_t::ICACTIVER0 => {
                AppleHypervisorGicRedistributorReg::ICACTIVER0
            }
            hv_gic_redistributor_reg_t::IPRIORITYR0 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR0
            }
            hv_gic_redistributor_reg_t::IPRIORITYR1 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR1
            }
            hv_gic_redistributor_reg_t::IPRIORITYR2 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR2
            }
            hv_gic_redistributor_reg_t::IPRIORITYR3 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR3
            }
            hv_gic_redistributor_reg_t::IPRIORITYR4 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR4
            }
            hv_gic_redistributor_reg_t::IPRIORITYR5 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR5
            }
            hv_gic_redistributor_reg_t::IPRIORITYR6 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR6
            }
            hv_gic_redistributor_reg_t::IPRIORITYR7 => {
                AppleHypervisorGicRedistributorReg::IPRIORITYR7
            }
            hv_gic_redistributor_reg_t::ICFGR0 => AppleHypervisorGicRedistributorReg::ICFGR0,
            hv_gic_redistributor_reg_t::ICFGR1 => AppleHypervisorGicRedistributorReg::ICFGR1,
        }
    }
}

impl From<AppleHypervisorGicRedistributorReg> for hv_gic_redistributor_reg_t {
    fn from(reg: AppleHypervisorGicRedistributorReg) -> Self {
        match reg {
            AppleHypervisorGicRedistributorReg::TYPER => hv_gic_redistributor_reg_t::TYPER,
            AppleHypervisorGicRedistributorReg::PIDR2 => hv_gic_redistributor_reg_t::PIDR2,
            AppleHypervisorGicRedistributorReg::IGROUPR0 => hv_gic_redistributor_reg_t::IGROUPR0,
            AppleHypervisorGicRedistributorReg::ISENABLER0 => {
                hv_gic_redistributor_reg_t::ISENABLER0
            }
            AppleHypervisorGicRedistributorReg::ICENABLER0 => {
                hv_gic_redistributor_reg_t::ICENABLER0
            }
            AppleHypervisorGicRedistributorReg::ISPENDR0 => hv_gic_redistributor_reg_t::ISPENDR0,
            AppleHypervisorGicRedistributorReg::ICPENDR0 => hv_gic_redistributor_reg_t::ICPENDR0,
            AppleHypervisorGicRedistributorReg::ISACTIVER0 => {
                hv_gic_redistributor_reg_t::ISACTIVER0
            }
            AppleHypervisorGicRedistributorReg::ICACTIVER0 => {
                hv_gic_redistributor_reg_t::ICACTIVER0
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR0 => {
                hv_gic_redistributor_reg_t::IPRIORITYR0
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR1 => {
                hv_gic_redistributor_reg_t::IPRIORITYR1
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR2 => {
                hv_gic_redistributor_reg_t::IPRIORITYR2
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR3 => {
                hv_gic_redistributor_reg_t::IPRIORITYR3
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR4 => {
                hv_gic_redistributor_reg_t::IPRIORITYR4
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR5 => {
                hv_gic_redistributor_reg_t::IPRIORITYR5
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR6 => {
                hv_gic_redistributor_reg_t::IPRIORITYR6
            }
            AppleHypervisorGicRedistributorReg::IPRIORITYR7 => {
                hv_gic_redistributor_reg_t::IPRIORITYR7
            }
            AppleHypervisorGicRedistributorReg::ICFGR0 => hv_gic_redistributor_reg_t::ICFGR0,
            AppleHypervisorGicRedistributorReg::ICFGR1 => hv_gic_redistributor_reg_t::ICFGR1,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorGicIccReg {
    PMR_EL1 = 0xc230,
    BPR0_EL1 = 0xc643,
    AP0R0_EL1 = 0xc644,
    AP1R0_EL1 = 0xc648,
    RPR_EL1 = 0xc65b,
    BPR1_EL1 = 0xc663,
    CTLR_EL1 = 0xc664,
    SRE_EL1 = 0xc665,
    IGRPEN0_EL1 = 0xc666,
    IGRPEN1_EL1 = 0xc667,
    SRE_EL2 = 0xe64d,
}

impl From<AppleHypervisorGicIccReg> for hv_gic_icc_reg_t {
    fn from(reg: AppleHypervisorGicIccReg) -> Self {
        match reg {
            AppleHypervisorGicIccReg::PMR_EL1 => hv_gic_icc_reg_t::PMR_EL1,
            AppleHypervisorGicIccReg::BPR0_EL1 => hv_gic_icc_reg_t::BPR0_EL1,
            AppleHypervisorGicIccReg::AP0R0_EL1 => hv_gic_icc_reg_t::AP0R0_EL1,
            AppleHypervisorGicIccReg::AP1R0_EL1 => hv_gic_icc_reg_t::AP1R0_EL1,
            AppleHypervisorGicIccReg::RPR_EL1 => hv_gic_icc_reg_t::RPR_EL1,
            AppleHypervisorGicIccReg::BPR1_EL1 => hv_gic_icc_reg_t::BPR1_EL1,
            AppleHypervisorGicIccReg::CTLR_EL1 => hv_gic_icc_reg_t::CTLR_EL1,
            AppleHypervisorGicIccReg::SRE_EL1 => hv_gic_icc_reg_t::SRE_EL1,
            AppleHypervisorGicIccReg::IGRPEN0_EL1 => hv_gic_icc_reg_t::IGRPEN0_EL1,
            AppleHypervisorGicIccReg::IGRPEN1_EL1 => hv_gic_icc_reg_t::IGRPEN1_EL1,
            AppleHypervisorGicIccReg::SRE_EL2 => hv_gic_icc_reg_t::SRE_EL2,
        }
    }
}

impl From<hv_gic_icc_reg_t> for AppleHypervisorGicIccReg {
    fn from(reg: hv_gic_icc_reg_t) -> Self {
        match reg {
            hv_gic_icc_reg_t::PMR_EL1 => AppleHypervisorGicIccReg::PMR_EL1,
            hv_gic_icc_reg_t::BPR0_EL1 => AppleHypervisorGicIccReg::BPR0_EL1,
            hv_gic_icc_reg_t::AP0R0_EL1 => AppleHypervisorGicIccReg::AP0R0_EL1,
            hv_gic_icc_reg_t::AP1R0_EL1 => AppleHypervisorGicIccReg::AP1R0_EL1,
            hv_gic_icc_reg_t::RPR_EL1 => AppleHypervisorGicIccReg::RPR_EL1,
            hv_gic_icc_reg_t::BPR1_EL1 => AppleHypervisorGicIccReg::BPR1_EL1,
            hv_gic_icc_reg_t::CTLR_EL1 => AppleHypervisorGicIccReg::CTLR_EL1,
            hv_gic_icc_reg_t::SRE_EL1 => AppleHypervisorGicIccReg::SRE_EL1,
            hv_gic_icc_reg_t::IGRPEN0_EL1 => AppleHypervisorGicIccReg::IGRPEN0_EL1,
            hv_gic_icc_reg_t::IGRPEN1_EL1 => AppleHypervisorGicIccReg::IGRPEN1_EL1,
            hv_gic_icc_reg_t::SRE_EL2 => AppleHypervisorGicIccReg::SRE_EL2,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorGicIchReg {
    AP0R0_EL2 = 0xe640,
    AP1R0_EL2 = 0xe648,
    HCR_EL2 = 0xe658,
    VTR_EL2 = 0xe659,
    MISR_EL2 = 0xe65a,
    EISR_EL2 = 0xe65b,
    ELRSR_EL2 = 0xe65d,
    VMCR_EL2 = 0xe65f,
    LR0_EL2 = 0xe660,
    LR1_EL2 = 0xe661,
    LR2_EL2 = 0xe662,
    LR3_EL2 = 0xe663,
    LR4_EL2 = 0xe664,
    LR5_EL2 = 0xe665,
    LR6_EL2 = 0xe666,
    LR7_EL2 = 0xe667,
    LR8_EL2 = 0xe668,
    LR9_EL2 = 0xe669,
    LR10_EL2 = 0xe66a,
    LR11_EL2 = 0xe66b,
    LR12_EL2 = 0xe66c,
    LR13_EL2 = 0xe66d,
    LR14_EL2 = 0xe66e,
    LR15_EL2 = 0xe66f,
}

impl From<AppleHypervisorGicIchReg> for hv_gic_ich_reg_t {
    fn from(reg: AppleHypervisorGicIchReg) -> Self {
        match reg {
            AppleHypervisorGicIchReg::AP0R0_EL2 => hv_gic_ich_reg_t::AP0R0_EL2,
            AppleHypervisorGicIchReg::AP1R0_EL2 => hv_gic_ich_reg_t::AP1R0_EL2,
            AppleHypervisorGicIchReg::HCR_EL2 => hv_gic_ich_reg_t::HCR_EL2,
            AppleHypervisorGicIchReg::VTR_EL2 => hv_gic_ich_reg_t::VTR_EL2,
            AppleHypervisorGicIchReg::MISR_EL2 => hv_gic_ich_reg_t::MISR_EL2,
            AppleHypervisorGicIchReg::EISR_EL2 => hv_gic_ich_reg_t::EISR_EL2,
            AppleHypervisorGicIchReg::ELRSR_EL2 => hv_gic_ich_reg_t::ELRSR_EL2,
            AppleHypervisorGicIchReg::VMCR_EL2 => hv_gic_ich_reg_t::VMCR_EL2,
            AppleHypervisorGicIchReg::LR0_EL2 => hv_gic_ich_reg_t::LR0_EL2,
            AppleHypervisorGicIchReg::LR1_EL2 => hv_gic_ich_reg_t::LR1_EL2,
            AppleHypervisorGicIchReg::LR2_EL2 => hv_gic_ich_reg_t::LR2_EL2,
            AppleHypervisorGicIchReg::LR3_EL2 => hv_gic_ich_reg_t::LR3_EL2,
            AppleHypervisorGicIchReg::LR4_EL2 => hv_gic_ich_reg_t::LR4_EL2,
            AppleHypervisorGicIchReg::LR5_EL2 => hv_gic_ich_reg_t::LR5_EL2,
            AppleHypervisorGicIchReg::LR6_EL2 => hv_gic_ich_reg_t::LR6_EL2,
            AppleHypervisorGicIchReg::LR7_EL2 => hv_gic_ich_reg_t::LR7_EL2,
            AppleHypervisorGicIchReg::LR8_EL2 => hv_gic_ich_reg_t::LR8_EL2,
            AppleHypervisorGicIchReg::LR9_EL2 => hv_gic_ich_reg_t::LR9_EL2,
            AppleHypervisorGicIchReg::LR10_EL2 => hv_gic_ich_reg_t::LR10_EL2,
            AppleHypervisorGicIchReg::LR11_EL2 => hv_gic_ich_reg_t::LR11_EL2,
            AppleHypervisorGicIchReg::LR12_EL2 => hv_gic_ich_reg_t::LR12_EL2,
            AppleHypervisorGicIchReg::LR13_EL2 => hv_gic_ich_reg_t::LR13_EL2,
            AppleHypervisorGicIchReg::LR14_EL2 => hv_gic_ich_reg_t::LR14_EL2,
            AppleHypervisorGicIchReg::LR15_EL2 => hv_gic_ich_reg_t::LR15_EL2,
        }
    }
}

impl From<hv_gic_ich_reg_t> for AppleHypervisorGicIchReg {
    fn from(reg: hv_gic_ich_reg_t) -> Self {
        match reg {
            hv_gic_ich_reg_t::AP0R0_EL2 => AppleHypervisorGicIchReg::AP0R0_EL2,
            hv_gic_ich_reg_t::AP1R0_EL2 => AppleHypervisorGicIchReg::AP1R0_EL2,
            hv_gic_ich_reg_t::HCR_EL2 => AppleHypervisorGicIchReg::HCR_EL2,
            hv_gic_ich_reg_t::VTR_EL2 => AppleHypervisorGicIchReg::VTR_EL2,
            hv_gic_ich_reg_t::MISR_EL2 => AppleHypervisorGicIchReg::MISR_EL2,
            hv_gic_ich_reg_t::EISR_EL2 => AppleHypervisorGicIchReg::EISR_EL2,
            hv_gic_ich_reg_t::ELRSR_EL2 => AppleHypervisorGicIchReg::ELRSR_EL2,
            hv_gic_ich_reg_t::VMCR_EL2 => AppleHypervisorGicIchReg::VMCR_EL2,
            hv_gic_ich_reg_t::LR0_EL2 => AppleHypervisorGicIchReg::LR0_EL2,
            hv_gic_ich_reg_t::LR1_EL2 => AppleHypervisorGicIchReg::LR1_EL2,
            hv_gic_ich_reg_t::LR2_EL2 => AppleHypervisorGicIchReg::LR2_EL2,
            hv_gic_ich_reg_t::LR3_EL2 => AppleHypervisorGicIchReg::LR3_EL2,
            hv_gic_ich_reg_t::LR4_EL2 => AppleHypervisorGicIchReg::LR4_EL2,
            hv_gic_ich_reg_t::LR5_EL2 => AppleHypervisorGicIchReg::LR5_EL2,
            hv_gic_ich_reg_t::LR6_EL2 => AppleHypervisorGicIchReg::LR6_EL2,
            hv_gic_ich_reg_t::LR7_EL2 => AppleHypervisorGicIchReg::LR7_EL2,
            hv_gic_ich_reg_t::LR8_EL2 => AppleHypervisorGicIchReg::LR8_EL2,
            hv_gic_ich_reg_t::LR9_EL2 => AppleHypervisorGicIchReg::LR9_EL2,
            hv_gic_ich_reg_t::LR10_EL2 => AppleHypervisorGicIchReg::LR10_EL2,
            hv_gic_ich_reg_t::LR11_EL2 => AppleHypervisorGicIchReg::LR11_EL2,
            hv_gic_ich_reg_t::LR12_EL2 => AppleHypervisorGicIchReg::LR12_EL2,
            hv_gic_ich_reg_t::LR13_EL2 => AppleHypervisorGicIchReg::LR13_EL2,
            hv_gic_ich_reg_t::LR14_EL2 => AppleHypervisorGicIchReg::LR14_EL2,
            hv_gic_ich_reg_t::LR15_EL2 => AppleHypervisorGicIchReg::LR15_EL2,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorGicIcvReg {
    PMR_EL1 = 0xc230,
    BPR0_EL1 = 0xc643,
    AP0R0_EL1 = 0xc644,
    AP1R0_EL1 = 0xc648,
    RPR_EL1 = 0xc65b,
    BPR1_EL1 = 0xc663,
    CTLR_EL1 = 0xc664,
    SRE_EL1 = 0xc665,
    IGRPEN0_EL1 = 0xc666,
    IGRPEN1_EL1 = 0xc667,
}

impl From<AppleHypervisorGicIcvReg> for hv_gic_icv_reg_t {
    fn from(reg: AppleHypervisorGicIcvReg) -> Self {
        match reg {
            AppleHypervisorGicIcvReg::PMR_EL1 => hv_gic_icv_reg_t::PMR_EL1,
            AppleHypervisorGicIcvReg::BPR0_EL1 => hv_gic_icv_reg_t::BPR0_EL1,
            AppleHypervisorGicIcvReg::AP0R0_EL1 => hv_gic_icv_reg_t::AP0R0_EL1,
            AppleHypervisorGicIcvReg::AP1R0_EL1 => hv_gic_icv_reg_t::AP1R0_EL1,
            AppleHypervisorGicIcvReg::RPR_EL1 => hv_gic_icv_reg_t::RPR_EL1,
            AppleHypervisorGicIcvReg::BPR1_EL1 => hv_gic_icv_reg_t::BPR1_EL1,
            AppleHypervisorGicIcvReg::CTLR_EL1 => hv_gic_icv_reg_t::CTLR_EL1,
            AppleHypervisorGicIcvReg::SRE_EL1 => hv_gic_icv_reg_t::SRE_EL1,
            AppleHypervisorGicIcvReg::IGRPEN0_EL1 => hv_gic_icv_reg_t::IGRPEN0_EL1,
            AppleHypervisorGicIcvReg::IGRPEN1_EL1 => hv_gic_icv_reg_t::IGRPEN1_EL1,
        }
    }
}

impl From<hv_gic_icv_reg_t> for AppleHypervisorGicIcvReg {
    fn from(reg: hv_gic_icv_reg_t) -> Self {
        match reg {
            hv_gic_icv_reg_t::PMR_EL1 => AppleHypervisorGicIcvReg::PMR_EL1,
            hv_gic_icv_reg_t::BPR0_EL1 => AppleHypervisorGicIcvReg::BPR0_EL1,
            hv_gic_icv_reg_t::AP0R0_EL1 => AppleHypervisorGicIcvReg::AP0R0_EL1,
            hv_gic_icv_reg_t::AP1R0_EL1 => AppleHypervisorGicIcvReg::AP1R0_EL1,
            hv_gic_icv_reg_t::RPR_EL1 => AppleHypervisorGicIcvReg::RPR_EL1,
            hv_gic_icv_reg_t::BPR1_EL1 => AppleHypervisorGicIcvReg::BPR1_EL1,
            hv_gic_icv_reg_t::CTLR_EL1 => AppleHypervisorGicIcvReg::CTLR_EL1,
            hv_gic_icv_reg_t::SRE_EL1 => AppleHypervisorGicIcvReg::SRE_EL1,
            hv_gic_icv_reg_t::IGRPEN0_EL1 => AppleHypervisorGicIcvReg::IGRPEN0_EL1,
            hv_gic_icv_reg_t::IGRPEN1_EL1 => AppleHypervisorGicIcvReg::IGRPEN1_EL1,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorCoreRegister {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    PC,
    FPCR,
    FPSR,
    CPSR,
}

impl From<AppleHypervisorCoreRegister> for hv_reg_t {
    fn from(reg: AppleHypervisorCoreRegister) -> Self {
        match reg {
            AppleHypervisorCoreRegister::X0 => hv_reg_t::X0,
            AppleHypervisorCoreRegister::X1 => hv_reg_t::X1,
            AppleHypervisorCoreRegister::X2 => hv_reg_t::X2,
            AppleHypervisorCoreRegister::X3 => hv_reg_t::X3,
            AppleHypervisorCoreRegister::X4 => hv_reg_t::X4,
            AppleHypervisorCoreRegister::X5 => hv_reg_t::X5,
            AppleHypervisorCoreRegister::X6 => hv_reg_t::X6,
            AppleHypervisorCoreRegister::X7 => hv_reg_t::X7,
            AppleHypervisorCoreRegister::X8 => hv_reg_t::X8,
            AppleHypervisorCoreRegister::X9 => hv_reg_t::X9,
            AppleHypervisorCoreRegister::X10 => hv_reg_t::X10,
            AppleHypervisorCoreRegister::X11 => hv_reg_t::X11,
            AppleHypervisorCoreRegister::X12 => hv_reg_t::X12,
            AppleHypervisorCoreRegister::X13 => hv_reg_t::X13,
            AppleHypervisorCoreRegister::X14 => hv_reg_t::X14,
            AppleHypervisorCoreRegister::X15 => hv_reg_t::X15,
            AppleHypervisorCoreRegister::X16 => hv_reg_t::X16,
            AppleHypervisorCoreRegister::X17 => hv_reg_t::X17,
            AppleHypervisorCoreRegister::X18 => hv_reg_t::X18,
            AppleHypervisorCoreRegister::X19 => hv_reg_t::X19,
            AppleHypervisorCoreRegister::X20 => hv_reg_t::X20,
            AppleHypervisorCoreRegister::X21 => hv_reg_t::X21,
            AppleHypervisorCoreRegister::X22 => hv_reg_t::X22,
            AppleHypervisorCoreRegister::X23 => hv_reg_t::X23,
            AppleHypervisorCoreRegister::X24 => hv_reg_t::X24,
            AppleHypervisorCoreRegister::X25 => hv_reg_t::X25,
            AppleHypervisorCoreRegister::X26 => hv_reg_t::X26,
            AppleHypervisorCoreRegister::X27 => hv_reg_t::X27,
            AppleHypervisorCoreRegister::X28 => hv_reg_t::X28,
            AppleHypervisorCoreRegister::X29 => hv_reg_t::X29,
            AppleHypervisorCoreRegister::X30 => hv_reg_t::X30,
            AppleHypervisorCoreRegister::PC => hv_reg_t::PC,
            AppleHypervisorCoreRegister::FPCR => hv_reg_t::FPCR,
            AppleHypervisorCoreRegister::FPSR => hv_reg_t::FPSR,
            AppleHypervisorCoreRegister::CPSR => hv_reg_t::CPSR,
        }
    }
}

impl From<hv_reg_t> for AppleHypervisorCoreRegister {
    fn from(reg: hv_reg_t) -> Self {
        match reg {
            hv_reg_t::X0 => AppleHypervisorCoreRegister::X0,
            hv_reg_t::X1 => AppleHypervisorCoreRegister::X1,
            hv_reg_t::X2 => AppleHypervisorCoreRegister::X2,
            hv_reg_t::X3 => AppleHypervisorCoreRegister::X3,
            hv_reg_t::X4 => AppleHypervisorCoreRegister::X4,
            hv_reg_t::X5 => AppleHypervisorCoreRegister::X5,
            hv_reg_t::X6 => AppleHypervisorCoreRegister::X6,
            hv_reg_t::X7 => AppleHypervisorCoreRegister::X7,
            hv_reg_t::X8 => AppleHypervisorCoreRegister::X8,
            hv_reg_t::X9 => AppleHypervisorCoreRegister::X9,
            hv_reg_t::X10 => AppleHypervisorCoreRegister::X10,
            hv_reg_t::X11 => AppleHypervisorCoreRegister::X11,
            hv_reg_t::X12 => AppleHypervisorCoreRegister::X12,
            hv_reg_t::X13 => AppleHypervisorCoreRegister::X13,
            hv_reg_t::X14 => AppleHypervisorCoreRegister::X14,
            hv_reg_t::X15 => AppleHypervisorCoreRegister::X15,
            hv_reg_t::X16 => AppleHypervisorCoreRegister::X16,
            hv_reg_t::X17 => AppleHypervisorCoreRegister::X17,
            hv_reg_t::X18 => AppleHypervisorCoreRegister::X18,
            hv_reg_t::X19 => AppleHypervisorCoreRegister::X19,
            hv_reg_t::X20 => AppleHypervisorCoreRegister::X20,
            hv_reg_t::X21 => AppleHypervisorCoreRegister::X21,
            hv_reg_t::X22 => AppleHypervisorCoreRegister::X22,
            hv_reg_t::X23 => AppleHypervisorCoreRegister::X23,
            hv_reg_t::X24 => AppleHypervisorCoreRegister::X24,
            hv_reg_t::X25 => AppleHypervisorCoreRegister::X25,
            hv_reg_t::X26 => AppleHypervisorCoreRegister::X26,
            hv_reg_t::X27 => AppleHypervisorCoreRegister::X27,
            hv_reg_t::X28 => AppleHypervisorCoreRegister::X28,
            hv_reg_t::X29 => AppleHypervisorCoreRegister::X29,
            hv_reg_t::X30 => AppleHypervisorCoreRegister::X30,
            hv_reg_t::PC => AppleHypervisorCoreRegister::PC,
            hv_reg_t::FPCR => AppleHypervisorCoreRegister::FPCR,
            hv_reg_t::FPSR => AppleHypervisorCoreRegister::FPSR,
            hv_reg_t::CPSR => AppleHypervisorCoreRegister::CPSR,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorSysRegister {
    DBGBVR0_EL1 = 0x8004,
    DBGBCR0_EL1 = 0x8005,
    DBGWVR0_EL1 = 0x8006,
    DBGWCR0_EL1 = 0x8007,
    DBGBVR1_EL1 = 0x800c,
    DBGBCR1_EL1 = 0x800d,
    DBGWVR1_EL1 = 0x800e,
    DBGWCR1_EL1 = 0x800f,
    MDCCINT_EL1 = 0x8010,
    MDSCR_EL1 = 0x8012,
    DBGBVR2_EL1 = 0x8014,
    DBGBCR2_EL1 = 0x8015,
    DBGWVR2_EL1 = 0x8016,
    DBGWCR2_EL1 = 0x8017,
    DBGBVR3_EL1 = 0x801c,
    DBGBCR3_EL1 = 0x801d,
    DBGWVR3_EL1 = 0x801e,
    DBGWCR3_EL1 = 0x801f,
    DBGBVR4_EL1 = 0x8024,
    DBGBCR4_EL1 = 0x8025,
    DBGWVR4_EL1 = 0x8026,
    DBGWCR4_EL1 = 0x8027,
    DBGBVR5_EL1 = 0x802c,
    DBGBCR5_EL1 = 0x802d,
    DBGWVR5_EL1 = 0x802e,
    DBGWCR5_EL1 = 0x802f,
    DBGBVR6_EL1 = 0x8034,
    DBGBCR6_EL1 = 0x8035,
    DBGWVR6_EL1 = 0x8036,
    DBGWCR6_EL1 = 0x8037,
    DBGBVR7_EL1 = 0x803c,
    DBGBCR7_EL1 = 0x803d,
    DBGWVR7_EL1 = 0x803e,
    DBGWCR7_EL1 = 0x803f,
    DBGBVR8_EL1 = 0x8044,
    DBGBCR8_EL1 = 0x8045,
    DBGWVR8_EL1 = 0x8046,
    DBGWCR8_EL1 = 0x8047,
    DBGBVR9_EL1 = 0x804c,
    DBGBCR9_EL1 = 0x804d,
    DBGWVR9_EL1 = 0x804e,
    DBGWCR9_EL1 = 0x804f,
    DBGBVR10_EL1 = 0x8054,
    DBGBCR10_EL1 = 0x8055,
    DBGWVR10_EL1 = 0x8056,
    DBGWCR10_EL1 = 0x8057,
    DBGBVR11_EL1 = 0x805c,
    DBGBCR11_EL1 = 0x805d,
    DBGWVR11_EL1 = 0x805e,
    DBGWCR11_EL1 = 0x805f,
    DBGBVR12_EL1 = 0x8064,
    DBGBCR12_EL1 = 0x8065,
    DBGWVR12_EL1 = 0x8066,
    DBGWCR12_EL1 = 0x8067,
    DBGBVR13_EL1 = 0x806c,
    DBGBCR13_EL1 = 0x806d,
    DBGWVR13_EL1 = 0x806e,
    DBGWCR13_EL1 = 0x806f,
    DBGBVR14_EL1 = 0x8074,
    DBGBCR14_EL1 = 0x8075,
    DBGWVR14_EL1 = 0x8076,
    DBGWCR14_EL1 = 0x8077,
    DBGBVR15_EL1 = 0x807c,
    DBGBCR15_EL1 = 0x807d,
    DBGWVR15_EL1 = 0x807e,
    DBGWCR15_EL1 = 0x807f,
    MIDR_EL1 = 0xc000,
    MPIDR_EL1 = 0xc005,
    ID_AA64PFR0_EL1 = 0xc020,
    ID_AA64PFR1_EL1 = 0xc021,
    ID_AA64ZFR0_EL1 = 0xc024,
    ID_AA64SMFR0_EL1 = 0xc025,
    ID_AA64DFR0_EL1 = 0xc028,
    ID_AA64DFR1_EL1 = 0xc029,
    ID_AA64ISAR0_EL1 = 0xc030,
    ID_AA64ISAR1_EL1 = 0xc031,
    ID_AA64MMFR0_EL1 = 0xc038,
    ID_AA64MMFR1_EL1 = 0xc039,
    ID_AA64MMFR2_EL1 = 0xc03a,
    SCTLR_EL1 = 0xc080,
    ACTLR_EL1 = 0xc081,
    CPACR_EL1 = 0xc082,
    SMPRI_EL1 = 0xc094,
    SMCR_EL1 = 0xc096,
    TTBR0_EL1 = 0xc100,
    TTBR1_EL1 = 0xc101,
    TCR_EL1 = 0xc102,
    APIAKEYLO_EL1 = 0xc108,
    APIAKEYHI_EL1 = 0xc109,
    APIBKEYLO_EL1 = 0xc10a,
    APIBKEYHI_EL1 = 0xc10b,
    APDAKEYLO_EL1 = 0xc110,
    APDAKEYHI_EL1 = 0xc111,
    APDBKEYLO_EL1 = 0xc112,
    APDBKEYHI_EL1 = 0xc113,
    APGAKEYLO_EL1 = 0xc118,
    APGAKEYHI_EL1 = 0xc119,
    SPSR_EL1 = 0xc200,
    ELR_EL1 = 0xc201,
    SP_EL0 = 0xc208,
    AFSR0_EL1 = 0xc288,
    AFSR1_EL1 = 0xc289,
    ESR_EL1 = 0xc290,
    FAR_EL1 = 0xc300,
    PAR_EL1 = 0xc3a0,
    MAIR_EL1 = 0xc510,
    AMAIR_EL1 = 0xc518,
    VBAR_EL1 = 0xc600,
    CONTEXTIDR_EL1 = 0xc681,
    TPIDR_EL1 = 0xc684,
    SCXTNUM_EL1 = 0xc687,
    CNTKCTL_EL1 = 0xc708,
    CSSELR_EL1 = 0xd000,
    TPIDR_EL0 = 0xde82,
    TPIDRRO_EL0 = 0xde83,
    TPIDR2_EL0 = 0xde85,
    SCXTNUM_EL0 = 0xde87,
    CNTV_CTL_EL0 = 0xdf19,
    CNTV_CVAL_EL0 = 0xdf1a,
    SP_EL1 = 0xe208,
    CNTP_TVAL_EL0 = 0xdf10,
    CNTP_CTL_EL0 = 0xdf11,
    CNTP_CVAL_EL0 = 0xdf12,
    CNTHCTL_EL2 = 0xe708,
    CNTHP_TVAL_EL2 = 0xe710,
    CNTHP_CTL_EL2 = 0xe711,
    CNTHP_CVAL_EL2 = 0xe712,
    CNTVOFF_EL2 = 0xe703,
    CPTR_EL2 = 0xe08a,
    ELR_EL2 = 0xe201,
    ESR_EL2 = 0xe290,
    FAR_EL2 = 0xe300,
    HCR_EL2 = 0xe088,
    HPFAR_EL2 = 0xe304,
    MAIR_EL2 = 0xe510,
    MDCR_EL2 = 0xe019,
    SCTLR_EL2 = 0xe080,
    SPSR_EL2 = 0xe200,
    SP_EL2 = 0xf208,
    TCR_EL2 = 0xe102,
    TPIDR_EL2 = 0xe682,
    TTBR0_EL2 = 0xe100,
    TTBR1_EL2 = 0xe101,
    VBAR_EL2 = 0xe600,
    VMPIDR_EL2 = 0xe005,
    VPIDR_EL2 = 0xe000,
    VTCR_EL2 = 0xe10a,
    VTTBR_EL2 = 0xe108,
}

impl TryFrom<SysRegister> for AppleHypervisorSysRegister {
    type Error = VcpuError;

    fn try_from(reg: SysRegister) -> Result<Self, Self::Error> {
        match reg {
            SysRegister::CnthctlEl2 => Ok(AppleHypervisorSysRegister::CNTHCTL_EL2),
            SysRegister::SctlrEl1 => Ok(AppleHypervisorSysRegister::SCTLR_EL1),
            SysRegister::TcrEl1 => Ok(AppleHypervisorSysRegister::TCR_EL1),
            SysRegister::Ttbr1El1 => Ok(AppleHypervisorSysRegister::TTBR1_EL1),
            SysRegister::MpidrEl1 => Ok(AppleHypervisorSysRegister::MPIDR_EL1),
            SysRegister::IdAa64mmfr0El1 => Ok(AppleHypervisorSysRegister::ID_AA64MMFR0_EL1),
            _ => Err(VcpuError::RegisterNotSupported(format!("{:?}", reg))),
        }
    }
}

impl From<AppleHypervisorSysRegister> for hv_sys_reg_t {
    fn from(reg: AppleHypervisorSysRegister) -> Self {
        match reg {
            AppleHypervisorSysRegister::DBGBVR0_EL1 => hv_sys_reg_t::DBGBVR0_EL1,
            AppleHypervisorSysRegister::DBGBCR0_EL1 => hv_sys_reg_t::DBGBCR0_EL1,
            AppleHypervisorSysRegister::DBGWVR0_EL1 => hv_sys_reg_t::DBGWVR0_EL1,
            AppleHypervisorSysRegister::DBGWCR0_EL1 => hv_sys_reg_t::DBGWCR0_EL1,
            AppleHypervisorSysRegister::DBGBVR1_EL1 => hv_sys_reg_t::DBGBVR1_EL1,
            AppleHypervisorSysRegister::DBGBCR1_EL1 => hv_sys_reg_t::DBGBCR1_EL1,
            AppleHypervisorSysRegister::DBGWVR1_EL1 => hv_sys_reg_t::DBGWVR1_EL1,
            AppleHypervisorSysRegister::DBGWCR1_EL1 => hv_sys_reg_t::DBGWCR1_EL1,
            AppleHypervisorSysRegister::MDCCINT_EL1 => hv_sys_reg_t::MDCCINT_EL1,
            AppleHypervisorSysRegister::MDSCR_EL1 => hv_sys_reg_t::MDSCR_EL1,
            AppleHypervisorSysRegister::DBGBVR2_EL1 => hv_sys_reg_t::DBGBVR2_EL1,
            AppleHypervisorSysRegister::DBGBCR2_EL1 => hv_sys_reg_t::DBGBCR2_EL1,
            AppleHypervisorSysRegister::DBGWVR2_EL1 => hv_sys_reg_t::DBGWVR2_EL1,
            AppleHypervisorSysRegister::DBGWCR2_EL1 => hv_sys_reg_t::DBGWCR2_EL1,
            AppleHypervisorSysRegister::DBGBVR3_EL1 => hv_sys_reg_t::DBGBVR3_EL1,
            AppleHypervisorSysRegister::DBGBCR3_EL1 => hv_sys_reg_t::DBGBCR3_EL1,
            AppleHypervisorSysRegister::DBGWVR3_EL1 => hv_sys_reg_t::DBGWVR3_EL1,
            AppleHypervisorSysRegister::DBGWCR3_EL1 => hv_sys_reg_t::DBGWCR3_EL1,
            AppleHypervisorSysRegister::DBGBVR4_EL1 => hv_sys_reg_t::DBGBVR4_EL1,
            AppleHypervisorSysRegister::DBGBCR4_EL1 => hv_sys_reg_t::DBGBCR4_EL1,
            AppleHypervisorSysRegister::DBGWVR4_EL1 => hv_sys_reg_t::DBGWVR4_EL1,
            AppleHypervisorSysRegister::DBGWCR4_EL1 => hv_sys_reg_t::DBGWCR4_EL1,
            AppleHypervisorSysRegister::DBGBVR5_EL1 => hv_sys_reg_t::DBGBVR5_EL1,
            AppleHypervisorSysRegister::DBGBCR5_EL1 => hv_sys_reg_t::DBGBCR5_EL1,
            AppleHypervisorSysRegister::DBGWVR5_EL1 => hv_sys_reg_t::DBGWVR5_EL1,
            AppleHypervisorSysRegister::DBGWCR5_EL1 => hv_sys_reg_t::DBGWCR5_EL1,
            AppleHypervisorSysRegister::DBGBVR6_EL1 => hv_sys_reg_t::DBGBVR6_EL1,
            AppleHypervisorSysRegister::DBGBCR6_EL1 => hv_sys_reg_t::DBGBCR6_EL1,
            AppleHypervisorSysRegister::DBGWVR6_EL1 => hv_sys_reg_t::DBGWVR6_EL1,
            AppleHypervisorSysRegister::DBGWCR6_EL1 => hv_sys_reg_t::DBGWCR6_EL1,
            AppleHypervisorSysRegister::DBGBVR7_EL1 => hv_sys_reg_t::DBGBVR7_EL1,
            AppleHypervisorSysRegister::DBGBCR7_EL1 => hv_sys_reg_t::DBGBCR7_EL1,
            AppleHypervisorSysRegister::DBGWVR7_EL1 => hv_sys_reg_t::DBGWVR7_EL1,
            AppleHypervisorSysRegister::DBGWCR7_EL1 => hv_sys_reg_t::DBGWCR7_EL1,
            AppleHypervisorSysRegister::DBGBVR8_EL1 => hv_sys_reg_t::DBGBVR8_EL1,
            AppleHypervisorSysRegister::DBGBCR8_EL1 => hv_sys_reg_t::DBGBCR8_EL1,
            AppleHypervisorSysRegister::DBGWVR8_EL1 => hv_sys_reg_t::DBGWVR8_EL1,
            AppleHypervisorSysRegister::DBGWCR8_EL1 => hv_sys_reg_t::DBGWCR8_EL1,
            AppleHypervisorSysRegister::DBGBVR9_EL1 => hv_sys_reg_t::DBGBVR9_EL1,
            AppleHypervisorSysRegister::DBGBCR9_EL1 => hv_sys_reg_t::DBGBCR9_EL1,
            AppleHypervisorSysRegister::DBGWVR9_EL1 => hv_sys_reg_t::DBGWVR9_EL1,
            AppleHypervisorSysRegister::DBGWCR9_EL1 => hv_sys_reg_t::DBGWCR9_EL1,
            AppleHypervisorSysRegister::DBGBVR10_EL1 => hv_sys_reg_t::DBGBVR10_EL1,
            AppleHypervisorSysRegister::DBGBCR10_EL1 => hv_sys_reg_t::DBGBCR10_EL1,
            AppleHypervisorSysRegister::DBGWVR10_EL1 => hv_sys_reg_t::DBGWVR10_EL1,
            AppleHypervisorSysRegister::DBGWCR10_EL1 => hv_sys_reg_t::DBGWCR10_EL1,
            AppleHypervisorSysRegister::DBGBVR11_EL1 => hv_sys_reg_t::DBGBVR11_EL1,
            AppleHypervisorSysRegister::DBGBCR11_EL1 => hv_sys_reg_t::DBGBCR11_EL1,
            AppleHypervisorSysRegister::DBGWVR11_EL1 => hv_sys_reg_t::DBGWVR11_EL1,
            AppleHypervisorSysRegister::DBGWCR11_EL1 => hv_sys_reg_t::DBGWCR11_EL1,
            AppleHypervisorSysRegister::DBGBVR12_EL1 => hv_sys_reg_t::DBGBVR12_EL1,
            AppleHypervisorSysRegister::DBGBCR12_EL1 => hv_sys_reg_t::DBGBCR12_EL1,
            AppleHypervisorSysRegister::DBGWVR12_EL1 => hv_sys_reg_t::DBGWVR12_EL1,
            AppleHypervisorSysRegister::DBGWCR12_EL1 => hv_sys_reg_t::DBGWCR12_EL1,
            AppleHypervisorSysRegister::DBGBVR13_EL1 => hv_sys_reg_t::DBGBVR13_EL1,
            AppleHypervisorSysRegister::DBGBCR13_EL1 => hv_sys_reg_t::DBGBCR13_EL1,
            AppleHypervisorSysRegister::DBGWVR13_EL1 => hv_sys_reg_t::DBGWVR13_EL1,
            AppleHypervisorSysRegister::DBGWCR13_EL1 => hv_sys_reg_t::DBGWCR13_EL1,
            AppleHypervisorSysRegister::DBGBVR14_EL1 => hv_sys_reg_t::DBGBVR14_EL1,
            AppleHypervisorSysRegister::DBGBCR14_EL1 => hv_sys_reg_t::DBGBCR14_EL1,
            AppleHypervisorSysRegister::DBGWVR14_EL1 => hv_sys_reg_t::DBGWVR14_EL1,
            AppleHypervisorSysRegister::DBGWCR14_EL1 => hv_sys_reg_t::DBGWCR14_EL1,
            AppleHypervisorSysRegister::DBGBVR15_EL1 => hv_sys_reg_t::DBGBVR15_EL1,
            AppleHypervisorSysRegister::DBGBCR15_EL1 => hv_sys_reg_t::DBGBCR15_EL1,
            AppleHypervisorSysRegister::DBGWVR15_EL1 => hv_sys_reg_t::DBGWVR15_EL1,
            AppleHypervisorSysRegister::DBGWCR15_EL1 => hv_sys_reg_t::DBGWCR15_EL1,
            AppleHypervisorSysRegister::MIDR_EL1 => hv_sys_reg_t::MIDR_EL1,
            AppleHypervisorSysRegister::MPIDR_EL1 => hv_sys_reg_t::MPIDR_EL1,
            AppleHypervisorSysRegister::ID_AA64PFR0_EL1 => hv_sys_reg_t::ID_AA64PFR0_EL1,
            AppleHypervisorSysRegister::ID_AA64PFR1_EL1 => hv_sys_reg_t::ID_AA64PFR1_EL1,
            AppleHypervisorSysRegister::ID_AA64ZFR0_EL1 => hv_sys_reg_t::ID_AA64ZFR0_EL1,
            AppleHypervisorSysRegister::ID_AA64SMFR0_EL1 => hv_sys_reg_t::ID_AA64SMFR0_EL1,
            AppleHypervisorSysRegister::ID_AA64DFR0_EL1 => hv_sys_reg_t::ID_AA64DFR0_EL1,
            AppleHypervisorSysRegister::ID_AA64DFR1_EL1 => hv_sys_reg_t::ID_AA64DFR1_EL1,
            AppleHypervisorSysRegister::ID_AA64ISAR0_EL1 => hv_sys_reg_t::ID_AA64ISAR0_EL1,
            AppleHypervisorSysRegister::ID_AA64ISAR1_EL1 => hv_sys_reg_t::ID_AA64ISAR1_EL1,
            AppleHypervisorSysRegister::ID_AA64MMFR0_EL1 => hv_sys_reg_t::ID_AA64MMFR0_EL1,
            AppleHypervisorSysRegister::ID_AA64MMFR1_EL1 => hv_sys_reg_t::ID_AA64MMFR1_EL1,
            AppleHypervisorSysRegister::ID_AA64MMFR2_EL1 => hv_sys_reg_t::ID_AA64MMFR2_EL1,
            AppleHypervisorSysRegister::SCTLR_EL1 => hv_sys_reg_t::SCTLR_EL1,
            AppleHypervisorSysRegister::ACTLR_EL1 => hv_sys_reg_t::ACTLR_EL1,
            AppleHypervisorSysRegister::CPACR_EL1 => hv_sys_reg_t::CPACR_EL1,
            AppleHypervisorSysRegister::SMPRI_EL1 => hv_sys_reg_t::SMPRI_EL1,
            AppleHypervisorSysRegister::SMCR_EL1 => hv_sys_reg_t::SMCR_EL1,
            AppleHypervisorSysRegister::TTBR0_EL1 => hv_sys_reg_t::TTBR0_EL1,
            AppleHypervisorSysRegister::TTBR1_EL1 => hv_sys_reg_t::TTBR1_EL1,
            AppleHypervisorSysRegister::TCR_EL1 => hv_sys_reg_t::TCR_EL1,
            AppleHypervisorSysRegister::APIAKEYLO_EL1 => hv_sys_reg_t::APIAKEYLO_EL1,
            AppleHypervisorSysRegister::APIAKEYHI_EL1 => hv_sys_reg_t::APIAKEYHI_EL1,
            AppleHypervisorSysRegister::APIBKEYLO_EL1 => hv_sys_reg_t::APIBKEYLO_EL1,
            AppleHypervisorSysRegister::APIBKEYHI_EL1 => hv_sys_reg_t::APIBKEYHI_EL1,
            AppleHypervisorSysRegister::APDAKEYLO_EL1 => hv_sys_reg_t::APDAKEYLO_EL1,
            AppleHypervisorSysRegister::APDAKEYHI_EL1 => hv_sys_reg_t::APDAKEYHI_EL1,
            AppleHypervisorSysRegister::APDBKEYLO_EL1 => hv_sys_reg_t::APDBKEYLO_EL1,
            AppleHypervisorSysRegister::APDBKEYHI_EL1 => hv_sys_reg_t::APDBKEYHI_EL1,
            AppleHypervisorSysRegister::APGAKEYLO_EL1 => hv_sys_reg_t::APGAKEYLO_EL1,
            AppleHypervisorSysRegister::APGAKEYHI_EL1 => hv_sys_reg_t::APGAKEYHI_EL1,
            AppleHypervisorSysRegister::SPSR_EL1 => hv_sys_reg_t::SPSR_EL1,
            AppleHypervisorSysRegister::ELR_EL1 => hv_sys_reg_t::ELR_EL1,
            AppleHypervisorSysRegister::SP_EL0 => hv_sys_reg_t::SP_EL0,
            AppleHypervisorSysRegister::AFSR0_EL1 => hv_sys_reg_t::AFSR0_EL1,
            AppleHypervisorSysRegister::AFSR1_EL1 => hv_sys_reg_t::AFSR1_EL1,
            AppleHypervisorSysRegister::ESR_EL1 => hv_sys_reg_t::ESR_EL1,
            AppleHypervisorSysRegister::FAR_EL1 => hv_sys_reg_t::FAR_EL1,
            AppleHypervisorSysRegister::PAR_EL1 => hv_sys_reg_t::PAR_EL1,
            AppleHypervisorSysRegister::MAIR_EL1 => hv_sys_reg_t::MAIR_EL1,
            AppleHypervisorSysRegister::AMAIR_EL1 => hv_sys_reg_t::AMAIR_EL1,
            AppleHypervisorSysRegister::VBAR_EL1 => hv_sys_reg_t::VBAR_EL1,
            AppleHypervisorSysRegister::CONTEXTIDR_EL1 => hv_sys_reg_t::CONTEXTIDR_EL1,
            AppleHypervisorSysRegister::TPIDR_EL1 => hv_sys_reg_t::TPIDR_EL1,
            AppleHypervisorSysRegister::SCXTNUM_EL1 => hv_sys_reg_t::SCXTNUM_EL1,
            AppleHypervisorSysRegister::CNTKCTL_EL1 => hv_sys_reg_t::CNTKCTL_EL1,
            AppleHypervisorSysRegister::CSSELR_EL1 => hv_sys_reg_t::CSSELR_EL1,
            AppleHypervisorSysRegister::TPIDR_EL0 => hv_sys_reg_t::TPIDR_EL0,
            AppleHypervisorSysRegister::TPIDRRO_EL0 => hv_sys_reg_t::TPIDRRO_EL0,
            AppleHypervisorSysRegister::TPIDR2_EL0 => hv_sys_reg_t::TPIDR2_EL0,
            AppleHypervisorSysRegister::SCXTNUM_EL0 => hv_sys_reg_t::SCXTNUM_EL0,
            AppleHypervisorSysRegister::CNTV_CTL_EL0 => hv_sys_reg_t::CNTV_CTL_EL0,
            AppleHypervisorSysRegister::CNTV_CVAL_EL0 => hv_sys_reg_t::CNTV_CVAL_EL0,
            AppleHypervisorSysRegister::SP_EL1 => hv_sys_reg_t::SP_EL1,
            AppleHypervisorSysRegister::CNTP_TVAL_EL0 => hv_sys_reg_t::CNTP_TVAL_EL0,
            AppleHypervisorSysRegister::CNTP_CTL_EL0 => hv_sys_reg_t::CNTP_CTL_EL0,
            AppleHypervisorSysRegister::CNTP_CVAL_EL0 => hv_sys_reg_t::CNTP_CVAL_EL0,
            AppleHypervisorSysRegister::CNTHCTL_EL2 => hv_sys_reg_t::CNTHCTL_EL2,
            AppleHypervisorSysRegister::CNTHP_TVAL_EL2 => hv_sys_reg_t::CNTHP_TVAL_EL2,
            AppleHypervisorSysRegister::CNTHP_CTL_EL2 => hv_sys_reg_t::CNTHP_CTL_EL2,
            AppleHypervisorSysRegister::CNTHP_CVAL_EL2 => hv_sys_reg_t::CNTHP_CVAL_EL2,
            AppleHypervisorSysRegister::CNTVOFF_EL2 => hv_sys_reg_t::CNTVOFF_EL2,
            AppleHypervisorSysRegister::CPTR_EL2 => hv_sys_reg_t::CPTR_EL2,
            AppleHypervisorSysRegister::ELR_EL2 => hv_sys_reg_t::ELR_EL2,
            AppleHypervisorSysRegister::ESR_EL2 => hv_sys_reg_t::ESR_EL2,
            AppleHypervisorSysRegister::FAR_EL2 => hv_sys_reg_t::FAR_EL2,
            AppleHypervisorSysRegister::HCR_EL2 => hv_sys_reg_t::HCR_EL2,
            AppleHypervisorSysRegister::HPFAR_EL2 => hv_sys_reg_t::HPFAR_EL2,
            AppleHypervisorSysRegister::MAIR_EL2 => hv_sys_reg_t::MAIR_EL2,
            AppleHypervisorSysRegister::MDCR_EL2 => hv_sys_reg_t::MDCR_EL2,
            AppleHypervisorSysRegister::SCTLR_EL2 => hv_sys_reg_t::SCTLR_EL2,
            AppleHypervisorSysRegister::SPSR_EL2 => hv_sys_reg_t::SPSR_EL2,
            AppleHypervisorSysRegister::SP_EL2 => hv_sys_reg_t::SP_EL2,
            AppleHypervisorSysRegister::TCR_EL2 => hv_sys_reg_t::TCR_EL2,
            AppleHypervisorSysRegister::TPIDR_EL2 => hv_sys_reg_t::TPIDR_EL2,
            AppleHypervisorSysRegister::TTBR0_EL2 => hv_sys_reg_t::TTBR0_EL2,
            AppleHypervisorSysRegister::TTBR1_EL2 => hv_sys_reg_t::TTBR1_EL2,
            AppleHypervisorSysRegister::VBAR_EL2 => hv_sys_reg_t::VBAR_EL2,
            AppleHypervisorSysRegister::VMPIDR_EL2 => hv_sys_reg_t::VMPIDR_EL2,
            AppleHypervisorSysRegister::VPIDR_EL2 => hv_sys_reg_t::VPIDR_EL2,
            AppleHypervisorSysRegister::VTCR_EL2 => hv_sys_reg_t::VTCR_EL2,
            AppleHypervisorSysRegister::VTTBR_EL2 => hv_sys_reg_t::VTTBR_EL2,
        }
    }
}

impl From<hv_sys_reg_t> for AppleHypervisorSysRegister {
    fn from(reg: hv_sys_reg_t) -> Self {
        match reg {
            hv_sys_reg_t::DBGBVR0_EL1 => AppleHypervisorSysRegister::DBGBVR0_EL1,
            hv_sys_reg_t::DBGBCR0_EL1 => AppleHypervisorSysRegister::DBGBCR0_EL1,
            hv_sys_reg_t::DBGWVR0_EL1 => AppleHypervisorSysRegister::DBGWVR0_EL1,
            hv_sys_reg_t::DBGWCR0_EL1 => AppleHypervisorSysRegister::DBGWCR0_EL1,
            hv_sys_reg_t::DBGBVR1_EL1 => AppleHypervisorSysRegister::DBGBVR1_EL1,
            hv_sys_reg_t::DBGBCR1_EL1 => AppleHypervisorSysRegister::DBGBCR1_EL1,
            hv_sys_reg_t::DBGWVR1_EL1 => AppleHypervisorSysRegister::DBGWVR1_EL1,
            hv_sys_reg_t::DBGWCR1_EL1 => AppleHypervisorSysRegister::DBGWCR1_EL1,
            hv_sys_reg_t::MDCCINT_EL1 => AppleHypervisorSysRegister::MDCCINT_EL1,
            hv_sys_reg_t::MDSCR_EL1 => AppleHypervisorSysRegister::MDSCR_EL1,
            hv_sys_reg_t::DBGBVR2_EL1 => AppleHypervisorSysRegister::DBGBVR2_EL1,
            hv_sys_reg_t::DBGBCR2_EL1 => AppleHypervisorSysRegister::DBGBCR2_EL1,
            hv_sys_reg_t::DBGWVR2_EL1 => AppleHypervisorSysRegister::DBGWVR2_EL1,
            hv_sys_reg_t::DBGWCR2_EL1 => AppleHypervisorSysRegister::DBGWCR2_EL1,
            hv_sys_reg_t::DBGBVR3_EL1 => AppleHypervisorSysRegister::DBGBVR3_EL1,
            hv_sys_reg_t::DBGBCR3_EL1 => AppleHypervisorSysRegister::DBGBCR3_EL1,
            hv_sys_reg_t::DBGWVR3_EL1 => AppleHypervisorSysRegister::DBGWVR3_EL1,
            hv_sys_reg_t::DBGWCR3_EL1 => AppleHypervisorSysRegister::DBGWCR3_EL1,
            hv_sys_reg_t::DBGBVR4_EL1 => AppleHypervisorSysRegister::DBGBVR4_EL1,
            hv_sys_reg_t::DBGBCR4_EL1 => AppleHypervisorSysRegister::DBGBCR4_EL1,
            hv_sys_reg_t::DBGWVR4_EL1 => AppleHypervisorSysRegister::DBGWVR4_EL1,
            hv_sys_reg_t::DBGWCR4_EL1 => AppleHypervisorSysRegister::DBGWCR4_EL1,
            hv_sys_reg_t::DBGBVR5_EL1 => AppleHypervisorSysRegister::DBGBVR5_EL1,
            hv_sys_reg_t::DBGBCR5_EL1 => AppleHypervisorSysRegister::DBGBCR5_EL1,
            hv_sys_reg_t::DBGWVR5_EL1 => AppleHypervisorSysRegister::DBGWVR5_EL1,
            hv_sys_reg_t::DBGWCR5_EL1 => AppleHypervisorSysRegister::DBGWCR5_EL1,
            hv_sys_reg_t::DBGBVR6_EL1 => AppleHypervisorSysRegister::DBGBVR6_EL1,
            hv_sys_reg_t::DBGBCR6_EL1 => AppleHypervisorSysRegister::DBGBCR6_EL1,
            hv_sys_reg_t::DBGWVR6_EL1 => AppleHypervisorSysRegister::DBGWVR6_EL1,
            hv_sys_reg_t::DBGWCR6_EL1 => AppleHypervisorSysRegister::DBGWCR6_EL1,
            hv_sys_reg_t::DBGBVR7_EL1 => AppleHypervisorSysRegister::DBGBVR7_EL1,
            hv_sys_reg_t::DBGBCR7_EL1 => AppleHypervisorSysRegister::DBGBCR7_EL1,
            hv_sys_reg_t::DBGWVR7_EL1 => AppleHypervisorSysRegister::DBGWVR7_EL1,
            hv_sys_reg_t::DBGWCR7_EL1 => AppleHypervisorSysRegister::DBGWCR7_EL1,
            hv_sys_reg_t::DBGBVR8_EL1 => AppleHypervisorSysRegister::DBGBVR8_EL1,
            hv_sys_reg_t::DBGBCR8_EL1 => AppleHypervisorSysRegister::DBGBCR8_EL1,
            hv_sys_reg_t::DBGWVR8_EL1 => AppleHypervisorSysRegister::DBGWVR8_EL1,
            hv_sys_reg_t::DBGWCR8_EL1 => AppleHypervisorSysRegister::DBGWCR8_EL1,
            hv_sys_reg_t::DBGBVR9_EL1 => AppleHypervisorSysRegister::DBGBVR9_EL1,
            hv_sys_reg_t::DBGBCR9_EL1 => AppleHypervisorSysRegister::DBGBCR9_EL1,
            hv_sys_reg_t::DBGWVR9_EL1 => AppleHypervisorSysRegister::DBGWVR9_EL1,
            hv_sys_reg_t::DBGWCR9_EL1 => AppleHypervisorSysRegister::DBGWCR9_EL1,
            hv_sys_reg_t::DBGBVR10_EL1 => AppleHypervisorSysRegister::DBGBVR10_EL1,
            hv_sys_reg_t::DBGBCR10_EL1 => AppleHypervisorSysRegister::DBGBCR10_EL1,
            hv_sys_reg_t::DBGWVR10_EL1 => AppleHypervisorSysRegister::DBGWVR10_EL1,
            hv_sys_reg_t::DBGWCR10_EL1 => AppleHypervisorSysRegister::DBGWCR10_EL1,
            hv_sys_reg_t::DBGBVR11_EL1 => AppleHypervisorSysRegister::DBGBVR11_EL1,
            hv_sys_reg_t::DBGBCR11_EL1 => AppleHypervisorSysRegister::DBGBCR11_EL1,
            hv_sys_reg_t::DBGWVR11_EL1 => AppleHypervisorSysRegister::DBGWVR11_EL1,
            hv_sys_reg_t::DBGWCR11_EL1 => AppleHypervisorSysRegister::DBGWCR11_EL1,
            hv_sys_reg_t::DBGBVR12_EL1 => AppleHypervisorSysRegister::DBGBVR12_EL1,
            hv_sys_reg_t::DBGBCR12_EL1 => AppleHypervisorSysRegister::DBGBCR12_EL1,
            hv_sys_reg_t::DBGWVR12_EL1 => AppleHypervisorSysRegister::DBGWVR12_EL1,
            hv_sys_reg_t::DBGWCR12_EL1 => AppleHypervisorSysRegister::DBGWCR12_EL1,
            hv_sys_reg_t::DBGBVR13_EL1 => AppleHypervisorSysRegister::DBGBVR13_EL1,
            hv_sys_reg_t::DBGBCR13_EL1 => AppleHypervisorSysRegister::DBGBCR13_EL1,
            hv_sys_reg_t::DBGWVR13_EL1 => AppleHypervisorSysRegister::DBGWVR13_EL1,
            hv_sys_reg_t::DBGWCR13_EL1 => AppleHypervisorSysRegister::DBGWCR13_EL1,
            hv_sys_reg_t::DBGBVR14_EL1 => AppleHypervisorSysRegister::DBGBVR14_EL1,
            hv_sys_reg_t::DBGBCR14_EL1 => AppleHypervisorSysRegister::DBGBCR14_EL1,
            hv_sys_reg_t::DBGWVR14_EL1 => AppleHypervisorSysRegister::DBGWVR14_EL1,
            hv_sys_reg_t::DBGWCR14_EL1 => AppleHypervisorSysRegister::DBGWCR14_EL1,
            hv_sys_reg_t::DBGBVR15_EL1 => AppleHypervisorSysRegister::DBGBVR15_EL1,
            hv_sys_reg_t::DBGBCR15_EL1 => AppleHypervisorSysRegister::DBGBCR15_EL1,
            hv_sys_reg_t::DBGWVR15_EL1 => AppleHypervisorSysRegister::DBGWVR15_EL1,
            hv_sys_reg_t::DBGWCR15_EL1 => AppleHypervisorSysRegister::DBGWCR15_EL1,
            hv_sys_reg_t::MIDR_EL1 => AppleHypervisorSysRegister::MIDR_EL1,
            hv_sys_reg_t::MPIDR_EL1 => AppleHypervisorSysRegister::MPIDR_EL1,
            hv_sys_reg_t::ID_AA64PFR0_EL1 => AppleHypervisorSysRegister::ID_AA64PFR0_EL1,
            hv_sys_reg_t::ID_AA64PFR1_EL1 => AppleHypervisorSysRegister::ID_AA64PFR1_EL1,
            hv_sys_reg_t::ID_AA64ZFR0_EL1 => AppleHypervisorSysRegister::ID_AA64ZFR0_EL1,
            hv_sys_reg_t::ID_AA64SMFR0_EL1 => AppleHypervisorSysRegister::ID_AA64SMFR0_EL1,
            hv_sys_reg_t::ID_AA64DFR0_EL1 => AppleHypervisorSysRegister::ID_AA64DFR0_EL1,
            hv_sys_reg_t::ID_AA64DFR1_EL1 => AppleHypervisorSysRegister::ID_AA64DFR1_EL1,
            hv_sys_reg_t::ID_AA64ISAR0_EL1 => AppleHypervisorSysRegister::ID_AA64ISAR0_EL1,
            hv_sys_reg_t::ID_AA64ISAR1_EL1 => AppleHypervisorSysRegister::ID_AA64ISAR1_EL1,
            hv_sys_reg_t::ID_AA64MMFR0_EL1 => AppleHypervisorSysRegister::ID_AA64MMFR0_EL1,
            hv_sys_reg_t::ID_AA64MMFR1_EL1 => AppleHypervisorSysRegister::ID_AA64MMFR1_EL1,
            hv_sys_reg_t::ID_AA64MMFR2_EL1 => AppleHypervisorSysRegister::ID_AA64MMFR2_EL1,
            hv_sys_reg_t::SCTLR_EL1 => AppleHypervisorSysRegister::SCTLR_EL1,
            hv_sys_reg_t::CPACR_EL1 => AppleHypervisorSysRegister::CPACR_EL1,
            hv_sys_reg_t::ACTLR_EL1 => AppleHypervisorSysRegister::ACTLR_EL1,
            hv_sys_reg_t::SMPRI_EL1 => AppleHypervisorSysRegister::SMPRI_EL1,
            hv_sys_reg_t::SMCR_EL1 => AppleHypervisorSysRegister::SMCR_EL1,
            hv_sys_reg_t::TTBR0_EL1 => AppleHypervisorSysRegister::TTBR0_EL1,
            hv_sys_reg_t::TTBR1_EL1 => AppleHypervisorSysRegister::TTBR1_EL1,
            hv_sys_reg_t::TCR_EL1 => AppleHypervisorSysRegister::TCR_EL1,
            hv_sys_reg_t::APIAKEYLO_EL1 => AppleHypervisorSysRegister::APIAKEYLO_EL1,
            hv_sys_reg_t::APIAKEYHI_EL1 => AppleHypervisorSysRegister::APIAKEYHI_EL1,
            hv_sys_reg_t::APIBKEYLO_EL1 => AppleHypervisorSysRegister::APIBKEYLO_EL1,
            hv_sys_reg_t::APIBKEYHI_EL1 => AppleHypervisorSysRegister::APIBKEYHI_EL1,
            hv_sys_reg_t::APDAKEYLO_EL1 => AppleHypervisorSysRegister::APDAKEYLO_EL1,
            hv_sys_reg_t::APDAKEYHI_EL1 => AppleHypervisorSysRegister::APDAKEYHI_EL1,
            hv_sys_reg_t::APDBKEYLO_EL1 => AppleHypervisorSysRegister::APDBKEYLO_EL1,
            hv_sys_reg_t::APDBKEYHI_EL1 => AppleHypervisorSysRegister::APDBKEYHI_EL1,
            hv_sys_reg_t::APGAKEYLO_EL1 => AppleHypervisorSysRegister::APGAKEYLO_EL1,
            hv_sys_reg_t::APGAKEYHI_EL1 => AppleHypervisorSysRegister::APGAKEYHI_EL1,
            hv_sys_reg_t::SPSR_EL1 => AppleHypervisorSysRegister::SPSR_EL1,
            hv_sys_reg_t::ELR_EL1 => AppleHypervisorSysRegister::ELR_EL1,
            hv_sys_reg_t::SP_EL0 => AppleHypervisorSysRegister::SP_EL0,
            hv_sys_reg_t::AFSR0_EL1 => AppleHypervisorSysRegister::AFSR0_EL1,
            hv_sys_reg_t::AFSR1_EL1 => AppleHypervisorSysRegister::AFSR1_EL1,
            hv_sys_reg_t::ESR_EL1 => AppleHypervisorSysRegister::ESR_EL1,
            hv_sys_reg_t::FAR_EL1 => AppleHypervisorSysRegister::FAR_EL1,
            hv_sys_reg_t::PAR_EL1 => AppleHypervisorSysRegister::PAR_EL1,
            hv_sys_reg_t::MAIR_EL1 => AppleHypervisorSysRegister::MAIR_EL1,
            hv_sys_reg_t::AMAIR_EL1 => AppleHypervisorSysRegister::AMAIR_EL1,
            hv_sys_reg_t::VBAR_EL1 => AppleHypervisorSysRegister::VBAR_EL1,
            hv_sys_reg_t::CONTEXTIDR_EL1 => AppleHypervisorSysRegister::CONTEXTIDR_EL1,
            hv_sys_reg_t::TPIDR_EL1 => AppleHypervisorSysRegister::TPIDR_EL1,
            hv_sys_reg_t::SCXTNUM_EL1 => AppleHypervisorSysRegister::SCXTNUM_EL1,
            hv_sys_reg_t::CNTKCTL_EL1 => AppleHypervisorSysRegister::CNTKCTL_EL1,
            hv_sys_reg_t::CSSELR_EL1 => AppleHypervisorSysRegister::CSSELR_EL1,
            hv_sys_reg_t::TPIDR_EL0 => AppleHypervisorSysRegister::TPIDR_EL0,
            hv_sys_reg_t::TPIDRRO_EL0 => AppleHypervisorSysRegister::TPIDRRO_EL0,
            hv_sys_reg_t::TPIDR2_EL0 => AppleHypervisorSysRegister::TPIDR2_EL0,
            hv_sys_reg_t::SCXTNUM_EL0 => AppleHypervisorSysRegister::SCXTNUM_EL0,
            hv_sys_reg_t::CNTV_CTL_EL0 => AppleHypervisorSysRegister::CNTV_CTL_EL0,
            hv_sys_reg_t::CNTV_CVAL_EL0 => AppleHypervisorSysRegister::CNTV_CVAL_EL0,
            hv_sys_reg_t::SP_EL1 => AppleHypervisorSysRegister::SP_EL1,
            hv_sys_reg_t::CNTP_CTL_EL0 => AppleHypervisorSysRegister::CNTP_CTL_EL0,
            hv_sys_reg_t::CNTP_CVAL_EL0 => AppleHypervisorSysRegister::CNTP_CVAL_EL0,
            hv_sys_reg_t::CNTP_TVAL_EL0 => AppleHypervisorSysRegister::CNTP_TVAL_EL0,
            hv_sys_reg_t::CNTHCTL_EL2 => AppleHypervisorSysRegister::CNTHCTL_EL2,
            hv_sys_reg_t::CNTHP_CTL_EL2 => AppleHypervisorSysRegister::CNTHP_CTL_EL2,
            hv_sys_reg_t::CNTHP_CVAL_EL2 => AppleHypervisorSysRegister::CNTHP_CVAL_EL2,
            hv_sys_reg_t::CNTHP_TVAL_EL2 => AppleHypervisorSysRegister::CNTHP_TVAL_EL2,
            hv_sys_reg_t::CNTVOFF_EL2 => AppleHypervisorSysRegister::CNTVOFF_EL2,
            hv_sys_reg_t::CPTR_EL2 => AppleHypervisorSysRegister::CPTR_EL2,
            hv_sys_reg_t::ELR_EL2 => AppleHypervisorSysRegister::ELR_EL2,
            hv_sys_reg_t::ESR_EL2 => AppleHypervisorSysRegister::ESR_EL2,
            hv_sys_reg_t::FAR_EL2 => AppleHypervisorSysRegister::FAR_EL2,
            hv_sys_reg_t::HCR_EL2 => AppleHypervisorSysRegister::HCR_EL2,
            hv_sys_reg_t::HPFAR_EL2 => AppleHypervisorSysRegister::HPFAR_EL2,
            hv_sys_reg_t::MAIR_EL2 => AppleHypervisorSysRegister::MAIR_EL2,
            hv_sys_reg_t::MDCR_EL2 => AppleHypervisorSysRegister::MDCR_EL2,
            hv_sys_reg_t::SCTLR_EL2 => AppleHypervisorSysRegister::SCTLR_EL2,
            hv_sys_reg_t::SPSR_EL2 => AppleHypervisorSysRegister::SPSR_EL2,
            hv_sys_reg_t::SP_EL2 => AppleHypervisorSysRegister::SP_EL2,
            hv_sys_reg_t::TCR_EL2 => AppleHypervisorSysRegister::TCR_EL2,
            hv_sys_reg_t::TPIDR_EL2 => AppleHypervisorSysRegister::TPIDR_EL2,
            hv_sys_reg_t::TTBR0_EL2 => AppleHypervisorSysRegister::TTBR0_EL2,
            hv_sys_reg_t::TTBR1_EL2 => AppleHypervisorSysRegister::TTBR1_EL2,
            hv_sys_reg_t::VBAR_EL2 => AppleHypervisorSysRegister::VBAR_EL2,
            hv_sys_reg_t::VMPIDR_EL2 => AppleHypervisorSysRegister::VMPIDR_EL2,
            hv_sys_reg_t::VPIDR_EL2 => AppleHypervisorSysRegister::VPIDR_EL2,
            hv_sys_reg_t::VTCR_EL2 => AppleHypervisorSysRegister::VTCR_EL2,
            hv_sys_reg_t::VTTBR_EL2 => AppleHypervisorSysRegister::VTTBR_EL2,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AppleHypervisorFpRegister {
    Q0,
    Q1,
    Q2,
    Q3,
    Q4,
    Q5,
    Q6,
    Q7,
    Q8,
    Q9,
    Q10,
    Q11,
    Q12,
    Q13,
    Q14,
    Q15,
    Q16,
    Q17,
    Q18,
    Q19,
    Q20,
    Q21,
    Q22,
    Q23,
    Q24,
    Q25,
    Q26,
    Q27,
    Q28,
    Q29,
    Q30,
    Q31,
}

impl From<FpRegister> for AppleHypervisorFpRegister {
    fn from(reg: FpRegister) -> Self {
        match reg {
            FpRegister::V0 => AppleHypervisorFpRegister::Q0,
            FpRegister::V1 => AppleHypervisorFpRegister::Q1,
            FpRegister::V2 => AppleHypervisorFpRegister::Q2,
            FpRegister::V3 => AppleHypervisorFpRegister::Q3,
            FpRegister::V4 => AppleHypervisorFpRegister::Q4,
            FpRegister::V5 => AppleHypervisorFpRegister::Q5,
            FpRegister::V6 => AppleHypervisorFpRegister::Q6,
            FpRegister::V7 => AppleHypervisorFpRegister::Q7,
            FpRegister::V8 => AppleHypervisorFpRegister::Q8,
            FpRegister::V9 => AppleHypervisorFpRegister::Q9,
            FpRegister::V10 => AppleHypervisorFpRegister::Q10,
            FpRegister::V11 => AppleHypervisorFpRegister::Q11,
            FpRegister::V12 => AppleHypervisorFpRegister::Q12,
            FpRegister::V13 => AppleHypervisorFpRegister::Q13,
            FpRegister::V14 => AppleHypervisorFpRegister::Q14,
            FpRegister::V15 => AppleHypervisorFpRegister::Q15,
            FpRegister::V16 => AppleHypervisorFpRegister::Q16,
            FpRegister::V17 => AppleHypervisorFpRegister::Q17,
            FpRegister::V18 => AppleHypervisorFpRegister::Q18,
            FpRegister::V19 => AppleHypervisorFpRegister::Q19,
            FpRegister::V20 => AppleHypervisorFpRegister::Q20,
            FpRegister::V21 => AppleHypervisorFpRegister::Q21,
            FpRegister::V22 => AppleHypervisorFpRegister::Q22,
            FpRegister::V23 => AppleHypervisorFpRegister::Q23,
            FpRegister::V24 => AppleHypervisorFpRegister::Q24,
            FpRegister::V25 => AppleHypervisorFpRegister::Q25,
            FpRegister::V26 => AppleHypervisorFpRegister::Q26,
            FpRegister::V27 => AppleHypervisorFpRegister::Q27,
            FpRegister::V28 => AppleHypervisorFpRegister::Q28,
            FpRegister::V29 => AppleHypervisorFpRegister::Q29,
            FpRegister::V30 => AppleHypervisorFpRegister::Q30,
            FpRegister::V31 => AppleHypervisorFpRegister::Q31,
        }
    }
}

impl From<AppleHypervisorFpRegister> for hv_simd_fp_reg_t {
    fn from(reg: AppleHypervisorFpRegister) -> Self {
        match reg {
            AppleHypervisorFpRegister::Q0 => hv_simd_fp_reg_t::Q0,
            AppleHypervisorFpRegister::Q1 => hv_simd_fp_reg_t::Q1,
            AppleHypervisorFpRegister::Q2 => hv_simd_fp_reg_t::Q2,
            AppleHypervisorFpRegister::Q3 => hv_simd_fp_reg_t::Q3,
            AppleHypervisorFpRegister::Q4 => hv_simd_fp_reg_t::Q4,
            AppleHypervisorFpRegister::Q5 => hv_simd_fp_reg_t::Q5,
            AppleHypervisorFpRegister::Q6 => hv_simd_fp_reg_t::Q6,
            AppleHypervisorFpRegister::Q7 => hv_simd_fp_reg_t::Q7,
            AppleHypervisorFpRegister::Q8 => hv_simd_fp_reg_t::Q8,
            AppleHypervisorFpRegister::Q9 => hv_simd_fp_reg_t::Q9,
            AppleHypervisorFpRegister::Q10 => hv_simd_fp_reg_t::Q10,
            AppleHypervisorFpRegister::Q11 => hv_simd_fp_reg_t::Q11,
            AppleHypervisorFpRegister::Q12 => hv_simd_fp_reg_t::Q12,
            AppleHypervisorFpRegister::Q13 => hv_simd_fp_reg_t::Q13,
            AppleHypervisorFpRegister::Q14 => hv_simd_fp_reg_t::Q14,
            AppleHypervisorFpRegister::Q15 => hv_simd_fp_reg_t::Q15,
            AppleHypervisorFpRegister::Q16 => hv_simd_fp_reg_t::Q16,
            AppleHypervisorFpRegister::Q17 => hv_simd_fp_reg_t::Q17,
            AppleHypervisorFpRegister::Q18 => hv_simd_fp_reg_t::Q18,
            AppleHypervisorFpRegister::Q19 => hv_simd_fp_reg_t::Q19,
            AppleHypervisorFpRegister::Q20 => hv_simd_fp_reg_t::Q20,
            AppleHypervisorFpRegister::Q21 => hv_simd_fp_reg_t::Q21,
            AppleHypervisorFpRegister::Q22 => hv_simd_fp_reg_t::Q22,
            AppleHypervisorFpRegister::Q23 => hv_simd_fp_reg_t::Q23,
            AppleHypervisorFpRegister::Q24 => hv_simd_fp_reg_t::Q24,
            AppleHypervisorFpRegister::Q25 => hv_simd_fp_reg_t::Q25,
            AppleHypervisorFpRegister::Q26 => hv_simd_fp_reg_t::Q26,
            AppleHypervisorFpRegister::Q27 => hv_simd_fp_reg_t::Q27,
            AppleHypervisorFpRegister::Q28 => hv_simd_fp_reg_t::Q28,
            AppleHypervisorFpRegister::Q29 => hv_simd_fp_reg_t::Q29,
            AppleHypervisorFpRegister::Q30 => hv_simd_fp_reg_t::Q30,
            AppleHypervisorFpRegister::Q31 => hv_simd_fp_reg_t::Q31,
        }
    }
}

impl From<hv_simd_fp_reg_t> for AppleHypervisorFpRegister {
    fn from(reg: hv_simd_fp_reg_t) -> Self {
        match reg {
            hv_simd_fp_reg_t::Q0 => AppleHypervisorFpRegister::Q0,
            hv_simd_fp_reg_t::Q1 => AppleHypervisorFpRegister::Q1,
            hv_simd_fp_reg_t::Q2 => AppleHypervisorFpRegister::Q2,
            hv_simd_fp_reg_t::Q3 => AppleHypervisorFpRegister::Q3,
            hv_simd_fp_reg_t::Q4 => AppleHypervisorFpRegister::Q4,
            hv_simd_fp_reg_t::Q5 => AppleHypervisorFpRegister::Q5,
            hv_simd_fp_reg_t::Q6 => AppleHypervisorFpRegister::Q6,
            hv_simd_fp_reg_t::Q7 => AppleHypervisorFpRegister::Q7,
            hv_simd_fp_reg_t::Q8 => AppleHypervisorFpRegister::Q8,
            hv_simd_fp_reg_t::Q9 => AppleHypervisorFpRegister::Q9,
            hv_simd_fp_reg_t::Q10 => AppleHypervisorFpRegister::Q10,
            hv_simd_fp_reg_t::Q11 => AppleHypervisorFpRegister::Q11,
            hv_simd_fp_reg_t::Q12 => AppleHypervisorFpRegister::Q12,
            hv_simd_fp_reg_t::Q13 => AppleHypervisorFpRegister::Q13,
            hv_simd_fp_reg_t::Q14 => AppleHypervisorFpRegister::Q14,
            hv_simd_fp_reg_t::Q15 => AppleHypervisorFpRegister::Q15,
            hv_simd_fp_reg_t::Q16 => AppleHypervisorFpRegister::Q16,
            hv_simd_fp_reg_t::Q17 => AppleHypervisorFpRegister::Q17,
            hv_simd_fp_reg_t::Q18 => AppleHypervisorFpRegister::Q18,
            hv_simd_fp_reg_t::Q19 => AppleHypervisorFpRegister::Q19,
            hv_simd_fp_reg_t::Q20 => AppleHypervisorFpRegister::Q20,
            hv_simd_fp_reg_t::Q21 => AppleHypervisorFpRegister::Q21,
            hv_simd_fp_reg_t::Q22 => AppleHypervisorFpRegister::Q22,
            hv_simd_fp_reg_t::Q23 => AppleHypervisorFpRegister::Q23,
            hv_simd_fp_reg_t::Q24 => AppleHypervisorFpRegister::Q24,
            hv_simd_fp_reg_t::Q25 => AppleHypervisorFpRegister::Q25,
            hv_simd_fp_reg_t::Q26 => AppleHypervisorFpRegister::Q26,
            hv_simd_fp_reg_t::Q27 => AppleHypervisorFpRegister::Q27,
            hv_simd_fp_reg_t::Q28 => AppleHypervisorFpRegister::Q28,
            hv_simd_fp_reg_t::Q29 => AppleHypervisorFpRegister::Q29,
            hv_simd_fp_reg_t::Q30 => AppleHypervisorFpRegister::Q30,
            hv_simd_fp_reg_t::Q31 => AppleHypervisorFpRegister::Q31,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppleHypervisorVcpuRegisters {
    pub gic_redistributor: BTreeMap<AppleHypervisorGicRedistributorReg, u64>,
    pub gic_icc: BTreeMap<AppleHypervisorGicIccReg, u64>,
    pub gic_ich: BTreeMap<AppleHypervisorGicIchReg, u64>,
    pub gic_icv: BTreeMap<AppleHypervisorGicIcvReg, u64>,
    pub core: BTreeMap<AppleHypervisorCoreRegister, u64>,
    pub sys: BTreeMap<AppleHypervisorSysRegister, u64>,
    pub fp: BTreeMap<AppleHypervisorFpRegister, u128>,
    pub vtimer_is_masked: bool,
    pub vtimer_offset: u64,
}
