use std::ptr::null_mut;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_exit_reason_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_sys_reg_t;
use applevisor_sys::hv_vcpu_create;
use applevisor_sys::hv_vcpu_exit_t;
use applevisor_sys::hv_vcpu_get_reg;
use applevisor_sys::hv_vcpu_get_sys_reg;
use applevisor_sys::hv_vcpu_run;
use applevisor_sys::hv_vcpu_set_reg;
use applevisor_sys::hv_vcpu_set_sys_reg;
use tracing::debug;
use tracing::trace;
use tracing::warn;

use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vcpu::reg::esr_el2;
use crate::arch::aarch64::vcpu::reg::esr_el2::EsrEl2;
use crate::arch::aarch64::vm_exit::VmExitReason;
use crate::cpu::error::VcpuError;
use crate::hypervisor::hvp::hv_unsafe_call;
use crate::hypervisor::vcpu::HypervisorVcpu;

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
            SysRegister::MpidrEl1 => hv_sys_reg_t::MPIDR_EL1,
            SysRegister::SctlrEl1 => hv_sys_reg_t::SCTLR_EL1,
            SysRegister::CnthctlEl2 => hv_sys_reg_t::CNTHCTL_EL2,
            SysRegister::OslarEl1 => todo!(),
            SysRegister::OslsrEl1 => todo!(),
            SysRegister::OsdlrEl1 => todo!(),
        }
    }
}

pub struct HvpVcpu {
    vcpu_id: usize,
    handler: Option<u64>,
    exit: Option<*const hv_vcpu_exit_t>,
}

unsafe impl Send for HvpVcpu {}

impl HvpVcpu {
    pub fn new(vcpu_id: usize) -> Self {
        HvpVcpu {
            vcpu_id,
            handler: None,
            exit: None,
        }
    }

    fn try_get_handler(&self) -> Result<u64, VcpuError> {
        self.handler
            .as_ref()
            .ok_or(VcpuError::VcpuNotCreated(self.vcpu_id))
            .copied()
    }

    fn try_get_exit_info(&self) -> Result<hv_vcpu_exit_t, VcpuError> {
        Ok(unsafe {
            **self
                .exit
                .as_ref()
                .ok_or(VcpuError::VcpuNotCreated(self.vcpu_id))?
        })
    }
}

impl AArch64Vcpu for HvpVcpu {
    fn get_core_reg(&mut self, reg: CoreRegister) -> Result<u64, VcpuError> {
        let handler = self.try_get_handler()?;

        let mut value = 0;

        match reg.to_hvp_reg() {
            HvpReg::CoreReg(reg) => hv_unsafe_call!(hv_vcpu_get_reg(handler, reg, &mut value))?,
            HvpReg::SysReg(reg) => hv_unsafe_call!(hv_vcpu_get_sys_reg(handler, reg, &mut value))?,
        }

        Ok(value)
    }

    fn set_core_reg(&mut self, reg: CoreRegister, value: u64) -> Result<(), VcpuError> {
        let handler = self.try_get_handler()?;

        match reg.to_hvp_reg() {
            HvpReg::CoreReg(reg) => {
                hv_unsafe_call!(hv_vcpu_set_reg(handler, reg, value)).map_err(Into::into)
            }
            HvpReg::SysReg(reg) => {
                hv_unsafe_call!(hv_vcpu_set_sys_reg(handler, reg, value)).map_err(Into::into)
            }
        }
    }

    fn get_sys_reg(&mut self, reg: SysRegister) -> Result<u64, VcpuError> {
        let handler = self.try_get_handler()?;

        let mut value = 0;

        hv_unsafe_call!(hv_vcpu_get_sys_reg(handler, reg.to_hvp_reg(), &mut value))?;

        Ok(value)
    }

    fn set_sys_reg(&mut self, reg: SysRegister, value: u64) -> Result<(), VcpuError> {
        let handler = self.try_get_handler()?;

        hv_unsafe_call!(hv_vcpu_set_sys_reg(handler, reg.to_hvp_reg(), value)).map_err(Into::into)
    }
}

impl HypervisorVcpu for HvpVcpu {
    fn post_init_within_thread(&mut self) -> Result<(), VcpuError> {
        let mut vcpu = 0;
        let mut exit = null_mut() as *const hv_vcpu_exit_t;
        hv_unsafe_call!(hv_vcpu_create(&mut vcpu, &mut exit, null_mut()))?;

        self.handler = Some(vcpu);
        self.exit = Some(exit);

        Ok(())
    }

    fn run(&mut self) -> Result<VmExitReason, VcpuError> {
        let handler = self.try_get_handler()?;

        hv_unsafe_call!(hv_vcpu_run(handler))?;

        let exit_info = self.try_get_exit_info()?;
        let pc = self.get_core_reg(CoreRegister::PC)?;
        trace!(pc, self.vcpu_id, ?exit_info, "vm exit");

        match exit_info.reason {
            hv_exit_reason_t::CANCELED => todo!(),
            hv_exit_reason_t::EXCEPTION => {
                let esr_el2 = EsrEl2::from(exit_info.exception.syndrome);
                let iss = esr_el2.iss();

                match esr_el2.ec()? {
                    esr_el2::Ec::Unknown => Ok(VmExitReason::Unknown),
                    esr_el2::Ec::Wf => Ok(VmExitReason::Wf),
                    esr_el2::Ec::Hvc => todo!(),
                    esr_el2::Ec::Smc => {
                        let imm16 = iss as u16;
                        if imm16 != 0 {
                            warn!("smc imm is not zero");
                        }
                        Ok(VmExitReason::Smc)
                    }
                    esr_el2::Ec::Trapped => {
                        let read = (iss & 0x1) != 0;
                        let crm = (iss >> 1) & 0xf;
                        let rt = (iss >> 5) & 0x1f;
                        let crn = (iss >> 10) & 0xf;
                        let op1 = (iss >> 14) & 0x7;
                        let op2 = (iss >> 17) & 0x7;
                        let op0 = (iss >> 20) & 0x3;
                        debug!(read, crm, rt, crn, op1, op2, op0);
                        let reg = SysRegister::decode(
                            op0 as u8, op1 as u8, crn as u8, crm as u8, op2 as u8,
                        );
                        if read {
                            todo!()
                        } else {
                            let data = if rt == 0b11111 {
                                0
                            } else {
                                self.get_core_reg(CoreRegister::from_srt(rt))?
                            };
                            Ok(VmExitReason::TrappedWrite { reg, data })
                        }
                    }
                    esr_el2::Ec::DA => {
                        let far_el2 = exit_info.exception.physical_address;

                        let is_write = (iss >> 6) & 0x1 != 0;
                        let len = match (iss >> 22) & 0x3 {
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
                            0 => self.get_core_reg(CoreRegister::X0),
                            1 => self.get_core_reg(CoreRegister::X1),
                            2 => self.get_core_reg(CoreRegister::X2),
                            3 => self.get_core_reg(CoreRegister::X3),
                            4 => self.get_core_reg(CoreRegister::X4),
                            5 => self.get_core_reg(CoreRegister::X5),
                            6 => self.get_core_reg(CoreRegister::X6),
                            19 => self.get_core_reg(CoreRegister::X19),
                            21 => self.get_core_reg(CoreRegister::X21),
                            22 => self.get_core_reg(CoreRegister::X22),
                            31 => Ok(0), // xzr
                            _ => unimplemented!("{srt}"),
                        }?;

                        if is_write {
                            Ok(VmExitReason::MMWrite {
                                gpa: far_el2,
                                buf: (data.to_le_bytes()[0..len]).to_vec(),
                                len,
                            })
                        } else {
                            Ok(VmExitReason::MMRead {
                                gpa: far_el2,
                                srt: CoreRegister::from_srt(srt),
                                len,
                            })
                        }
                    }
                }
            }
            hv_exit_reason_t::VTIMER_ACTIVATED => todo!(),
            hv_exit_reason_t::UNKNOWN => Ok(VmExitReason::Unknown),
        }
    }
}
