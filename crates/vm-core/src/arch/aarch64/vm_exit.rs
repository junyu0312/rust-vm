use tracing::trace;

use crate::arch::aarch64::firmware::psci::error::PsciError;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::device_manager::vm_exit::DeviceError;
use crate::vcpu::error::VcpuError;
use crate::vcpu::vcpu::Vcpu;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle mmio, err: {0}")]
    DeviceError(#[from] DeviceError),

    #[error("Failed to rw vcpu, err: {0}")]
    VcpuError(String),

    #[error("Failed to handle smc, err: {0}")]
    PsciError(#[from] PsciError),
}

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
    vcpu: &mut Vcpu,
    exit_reason: VmExitReason,
) -> Result<HandleVmExitResult, VcpuError> {
    let device_vm_exit_handler = vcpu.device_vm_exit_handler.clone();

    trace!(?exit_reason);

    match exit_reason {
        VmExitReason::Unknown => Ok(HandleVmExitResult::Continue),
        VmExitReason::Wf => Ok(HandleVmExitResult::NextInstruction),
        VmExitReason::MMRead { gpa, srt, len } if device_vm_exit_handler.in_mmio_region(gpa) => {
            let mut buf = [0; 8];
            device_vm_exit_handler.mmio_read(gpa, len, &mut buf[0..len])?;
            vcpu.vcpu_instance
                .set_core_reg(srt, u64::from_le_bytes(buf))?;
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::MMRead { .. } => todo!(),
        VmExitReason::MMWrite { gpa, buf, len } if device_vm_exit_handler.in_mmio_region(gpa) => {
            device_vm_exit_handler.mmio_write(gpa, len, &buf)?;
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
            let psci = vcpu.psci.clone();

            // We only support psci for smc now
            psci.call(vcpu.vcpu_instance.as_mut())?;

            Ok(HandleVmExitResult::NextInstruction)
        }
    }
}
