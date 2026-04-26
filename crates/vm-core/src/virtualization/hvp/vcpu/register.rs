use applevisor_sys::hv_error_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_simd_fp_reg_t;
use applevisor_sys::hv_sys_reg_t;
use applevisor_sys::hv_vcpu_get_reg;

use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::FpRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::cpu::error::VcpuError;
use crate::virtualization::hvp::hv_unsafe_call;

pub enum HvpReg {
    CoreReg(hv_reg_t),
    SysReg(hv_sys_reg_t),
}

impl From<CoreRegister> for HvpReg {
    fn from(reg: CoreRegister) -> Self {
        match reg {
            CoreRegister::X0 => HvpReg::CoreReg(hv_reg_t::X0),
            CoreRegister::X1 => HvpReg::CoreReg(hv_reg_t::X1),
            CoreRegister::X2 => HvpReg::CoreReg(hv_reg_t::X2),
            CoreRegister::X3 => HvpReg::CoreReg(hv_reg_t::X3),
            CoreRegister::X4 => HvpReg::CoreReg(hv_reg_t::X4),
            CoreRegister::X5 => HvpReg::CoreReg(hv_reg_t::X5),
            CoreRegister::X6 => HvpReg::CoreReg(hv_reg_t::X6),
            CoreRegister::X7 => HvpReg::CoreReg(hv_reg_t::X7),
            CoreRegister::X8 => HvpReg::CoreReg(hv_reg_t::X8),
            CoreRegister::X9 => HvpReg::CoreReg(hv_reg_t::X9),
            CoreRegister::X10 => HvpReg::CoreReg(hv_reg_t::X10),
            CoreRegister::X11 => HvpReg::CoreReg(hv_reg_t::X11),
            CoreRegister::X12 => HvpReg::CoreReg(hv_reg_t::X12),
            CoreRegister::X13 => HvpReg::CoreReg(hv_reg_t::X13),
            CoreRegister::X14 => HvpReg::CoreReg(hv_reg_t::X14),
            CoreRegister::X15 => HvpReg::CoreReg(hv_reg_t::X15),
            CoreRegister::X16 => HvpReg::CoreReg(hv_reg_t::X16),
            CoreRegister::X17 => HvpReg::CoreReg(hv_reg_t::X17),
            CoreRegister::X18 => HvpReg::CoreReg(hv_reg_t::X18),
            CoreRegister::X19 => HvpReg::CoreReg(hv_reg_t::X19),
            CoreRegister::X20 => HvpReg::CoreReg(hv_reg_t::X20),
            CoreRegister::X21 => HvpReg::CoreReg(hv_reg_t::X21),
            CoreRegister::X22 => HvpReg::CoreReg(hv_reg_t::X22),
            CoreRegister::X23 => HvpReg::CoreReg(hv_reg_t::X23),
            CoreRegister::X24 => HvpReg::CoreReg(hv_reg_t::X24),
            CoreRegister::X25 => HvpReg::CoreReg(hv_reg_t::X25),
            CoreRegister::X26 => HvpReg::CoreReg(hv_reg_t::X26),
            CoreRegister::X27 => HvpReg::CoreReg(hv_reg_t::X27),
            CoreRegister::X28 => HvpReg::CoreReg(hv_reg_t::X28),
            CoreRegister::X29 => HvpReg::CoreReg(hv_reg_t::X29),
            CoreRegister::X30 => HvpReg::CoreReg(hv_reg_t::X30),
            CoreRegister::SP => HvpReg::SysReg(hv_sys_reg_t::SP_EL0),
            CoreRegister::PC => HvpReg::CoreReg(hv_reg_t::PC),
            CoreRegister::PState => HvpReg::CoreReg(hv_reg_t::CPSR),
            CoreRegister::Fpcr => HvpReg::CoreReg(hv_reg_t::FPCR),
            CoreRegister::Fpsr => HvpReg::CoreReg(hv_reg_t::FPSR),
        }
    }
}

impl From<FpRegister> for hv_simd_fp_reg_t {
    fn from(reg: FpRegister) -> Self {
        match reg {
            FpRegister::V0 => hv_simd_fp_reg_t::Q0,
            FpRegister::V1 => hv_simd_fp_reg_t::Q1,
            FpRegister::V2 => hv_simd_fp_reg_t::Q2,
            FpRegister::V3 => hv_simd_fp_reg_t::Q3,
            FpRegister::V4 => hv_simd_fp_reg_t::Q4,
            FpRegister::V5 => hv_simd_fp_reg_t::Q5,
            FpRegister::V6 => hv_simd_fp_reg_t::Q6,
            FpRegister::V7 => hv_simd_fp_reg_t::Q7,
            FpRegister::V8 => hv_simd_fp_reg_t::Q8,
            FpRegister::V9 => hv_simd_fp_reg_t::Q9,
            FpRegister::V10 => hv_simd_fp_reg_t::Q10,
            FpRegister::V11 => hv_simd_fp_reg_t::Q11,
            FpRegister::V12 => hv_simd_fp_reg_t::Q12,
            FpRegister::V13 => hv_simd_fp_reg_t::Q13,
            FpRegister::V14 => hv_simd_fp_reg_t::Q14,
            FpRegister::V15 => hv_simd_fp_reg_t::Q15,
            FpRegister::V16 => hv_simd_fp_reg_t::Q16,
            FpRegister::V17 => hv_simd_fp_reg_t::Q17,
            FpRegister::V18 => hv_simd_fp_reg_t::Q18,
            FpRegister::V19 => hv_simd_fp_reg_t::Q19,
            FpRegister::V20 => hv_simd_fp_reg_t::Q20,
            FpRegister::V21 => hv_simd_fp_reg_t::Q21,
            FpRegister::V22 => hv_simd_fp_reg_t::Q22,
            FpRegister::V23 => hv_simd_fp_reg_t::Q23,
            FpRegister::V24 => hv_simd_fp_reg_t::Q24,
            FpRegister::V25 => hv_simd_fp_reg_t::Q25,
            FpRegister::V26 => hv_simd_fp_reg_t::Q26,
            FpRegister::V27 => hv_simd_fp_reg_t::Q27,
            FpRegister::V28 => hv_simd_fp_reg_t::Q28,
            FpRegister::V29 => hv_simd_fp_reg_t::Q29,
            FpRegister::V30 => hv_simd_fp_reg_t::Q30,
            FpRegister::V31 => hv_simd_fp_reg_t::Q31,
        }
    }
}

impl From<SysRegister> for hv_sys_reg_t {
    fn from(reg: SysRegister) -> Self {
        match reg {
            SysRegister::MpidrEl1 => hv_sys_reg_t::MPIDR_EL1,
            SysRegister::SctlrEl1 => hv_sys_reg_t::SCTLR_EL1,
            SysRegister::CnthctlEl2 => hv_sys_reg_t::CNTHCTL_EL2,
            SysRegister::OslarEl1 => todo!(),
            SysRegister::OslsrEl1 => todo!(),
            SysRegister::OsdlrEl1 => todo!(),
            SysRegister::TcrEl1 => hv_sys_reg_t::TCR_EL1,
            SysRegister::Ttbr1El1 => hv_sys_reg_t::TTBR1_EL1,
            SysRegister::IdAa64mmfr0El1 => hv_sys_reg_t::ID_AA64MMFR0_EL1,
        }
    }
}

pub unsafe fn get_reg(vcpu: u64, rt: u64) -> Result<u64, VcpuError> {
    let mut data = 0;

    if rt != 31 {
        let reg = match rt {
            0 => hv_reg_t::X0,
            1 => hv_reg_t::X1,
            2 => hv_reg_t::X2,
            3 => hv_reg_t::X3,
            4 => hv_reg_t::X4,
            5 => hv_reg_t::X5,
            6 => hv_reg_t::X6,
            19 => hv_reg_t::X19,
            21 => hv_reg_t::X21,
            22 => hv_reg_t::X22,
            _ => unimplemented!("{rt}"),
        };
        hv_unsafe_call!(hv_vcpu_get_reg(vcpu, reg, &mut data))?;
    };

    Ok(data)
}
