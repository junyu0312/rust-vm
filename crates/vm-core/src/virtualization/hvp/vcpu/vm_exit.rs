use applevisor_sys::hv_exit_reason_t;
use applevisor_sys::hv_vcpu_exit_t;
use tracing::debug;
use tracing::warn;

use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vcpu::reg::esr_el2::EsrEl2;
use crate::arch::aarch64::vcpu::reg::esr_el2::{self};
use crate::arch::aarch64::vm_exit::VmExitReason;
use crate::cpu::error::VcpuError;
use crate::virtualization::hvp::vcpu::register::get_reg;

pub fn to_vm_exit(vcpu: u64, exit_info: hv_vcpu_exit_t) -> Result<VmExitReason, VcpuError> {
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
                    let reg =
                        SysRegister::decode(op0 as u8, op1 as u8, crn as u8, crm as u8, op2 as u8);
                    if read {
                        todo!()
                    } else {
                        let data = if rt == 0b11111 {
                            0
                        } else {
                            unsafe { get_reg(vcpu, rt) }?
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

                    let data = unsafe { get_reg(vcpu, srt) }?;

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
