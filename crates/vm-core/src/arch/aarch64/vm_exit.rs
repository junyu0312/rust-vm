use tracing::trace;

use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vcpu::error::VcpuError;

#[derive(Debug)]
pub enum VmExitReason {
    Unknown,
    Wf,
    MMRead {
        gpa: u64,
        srt: CoreRegister,
        len: usize,
    },
    MMWrite {
        gpa: u64,
        buf: Vec<u8>,
        len: usize,
    },
    TrappedRead {
        reg: SysRegister,
        rt: CoreRegister,
    },
    TrappedWrite {
        reg: SysRegister,
        data: u64,
    },
    Smc,
}

pub enum HandleVmExitResult {
    Continue,
    NextInstruction,
}

pub fn handle_vm_exit(
    vcpu: &mut dyn AArch64Vcpu,
    exit_reason: VmExitReason,
    vm_exit_handler: &dyn VmExit,
) -> Result<HandleVmExitResult, VcpuError> {
    trace!(?exit_reason);

    match exit_reason {
        VmExitReason::Unknown => Ok(HandleVmExitResult::Continue),
        VmExitReason::Wf => Ok(HandleVmExitResult::NextInstruction),
        VmExitReason::MMRead { gpa, srt, len } if vm_exit_handler.in_mmio_region(gpa) => {
            let mut buf = [0; 8];
            vm_exit_handler.mmio_read(gpa, len, &mut buf[0..len])?;
            vcpu.set_core_reg(srt, u64::from_le_bytes(buf))?;
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::MMRead { .. } => todo!(),
        VmExitReason::MMWrite { gpa, buf, len } if vm_exit_handler.in_mmio_region(gpa) => {
            vm_exit_handler.mmio_write(gpa, len, &buf)?;
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::MMWrite { .. } => todo!(),
        VmExitReason::TrappedRead { .. } => todo!(),
        VmExitReason::TrappedWrite { reg, .. } => {
            match reg {
                SysRegister::OslarEl1 => (), // TODO
                SysRegister::OsdlrEl1 => (), // TODO
                _ => unimplemented!(),
            }
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::Smc => {
            // We only support psci for smc now
            vm_exit_handler.call_smc(vcpu)?;

            Ok(HandleVmExitResult::NextInstruction)
        }
    }
}
