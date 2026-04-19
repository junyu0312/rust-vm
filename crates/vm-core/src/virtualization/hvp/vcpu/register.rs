use applevisor_sys::hv_error_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_sys_reg_t;
use applevisor_sys::hv_vcpu_get_reg;

use crate::arch::aarch64::vcpu::reg::CoreRegister;
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
