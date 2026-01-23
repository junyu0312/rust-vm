use applevisor_sys::hv_exit_reason_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_sys_reg_t;
use tracing::trace;

use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::vcpu::arch::aarch64::reg::SysRegister;
use crate::vcpu::arch::aarch64::reg::esr_el2;
use crate::vcpu::arch::aarch64::reg::esr_el2::EsrEl2;
use crate::virt::vm_exit::VmExitReason;

enum HvpReg {
    CoreReg(hv_reg_t),
    SysReg(hv_sys_reg_t),
}

impl CoreRegister {
    fn to_hvp_reg(&self) -> HvpReg {
        match self {
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

impl SysRegister {
    fn to_hvp_reg(&self) -> hv_sys_reg_t {
        match self {
            SysRegister::SctlrEl1 => hv_sys_reg_t::SCTLR_EL1,
            SysRegister::CnthctlEl2 => hv_sys_reg_t::CNTHCTL_EL2,
        }
    }
}

pub struct HvpVcpu {
    vcpu_id: u64,
    vcpu: applevisor::vcpu::Vcpu,
}

impl HvpVcpu {
    pub fn new(vcpu_id: u64, vcpu: applevisor::vcpu::Vcpu) -> Self {
        HvpVcpu { vcpu_id, vcpu }
    }
}

impl AArch64Vcpu for HvpVcpu {
    fn get_core_reg(&self, reg: CoreRegister) -> anyhow::Result<u64> {
        match reg.to_hvp_reg() {
            HvpReg::CoreReg(reg) => Ok(self.vcpu.get_reg(reg)?),
            HvpReg::SysReg(reg) => Ok(self.vcpu.get_sys_reg(reg)?),
        }
    }

    fn set_core_reg(&self, reg: CoreRegister, value: u64) -> anyhow::Result<()> {
        match reg.to_hvp_reg() {
            HvpReg::CoreReg(reg) => Ok(self.vcpu.set_reg(reg, value)?),
            HvpReg::SysReg(reg) => Ok(self.vcpu.set_sys_reg(reg, value)?),
        }
    }

    fn get_sys_reg(&self, reg: SysRegister) -> anyhow::Result<u64> {
        Ok(self.vcpu.get_sys_reg(reg.to_hvp_reg())?)
    }

    fn set_sys_reg(&self, reg: SysRegister, value: u64) -> anyhow::Result<()> {
        self.vcpu.set_sys_reg(reg.to_hvp_reg(), value)?;

        Ok(())
    }
}

impl Vcpu for HvpVcpu {
    fn run(&mut self) -> anyhow::Result<VmExitReason> {
        self.vcpu.run()?;

        let exit_info = self.vcpu.get_exit_info();

        trace!(self.vcpu_id, ?exit_info, "vm exit");

        match exit_info.reason {
            hv_exit_reason_t::CANCELED => todo!(),
            hv_exit_reason_t::EXCEPTION => {
                let esr_el2 = EsrEl2::from(exit_info.exception.syndrome);
                match esr_el2.ec()? {
                    esr_el2::Ec::Unknown => todo!(),
                    esr_el2::Ec::DA => {
                        let far_el2 = exit_info.exception.physical_address;
                        let is_write = (esr_el2.iss() >> 6) & 0x1 != 0;
                        let len = match (esr_el2.iss() >> 22) & 0x3 {
                            0 => 1,
                            1 => 2,
                            2 => 4,
                            3 => 8,
                            _ => unreachable!(),
                        };
                        let isv = (esr_el2.iss() >> 24) & 0x1 != 0;
                        let srt = if isv {
                            (esr_el2.iss() >> 16) & 0x1f
                        } else {
                            todo!()
                        };
                        let data = match srt {
                            0 => self.vcpu.get_reg(hv_reg_t::X0),
                            1 => self.vcpu.get_reg(hv_reg_t::X1),
                            2 => self.vcpu.get_reg(hv_reg_t::X2),
                            _ => unimplemented!("{srt}"),
                        }?;

                        let data = if is_write { Some(data) } else { None };
                        let il = esr_el2.il();
                        Ok(VmExitReason::MMIO {
                            gpa: far_el2,
                            data,
                            is_write,
                            len,
                            is_32bit_inst: il,
                        })
                    }
                }
            }
            hv_exit_reason_t::VTIMER_ACTIVATED => todo!(),
            hv_exit_reason_t::UNKNOWN => Ok(VmExitReason::Unknown),
        }
    }
}
