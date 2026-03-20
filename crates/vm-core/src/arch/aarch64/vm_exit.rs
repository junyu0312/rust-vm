use tracing::trace;

use crate::arch::aarch64::firmware::psci::Psci;
use crate::arch::aarch64::firmware::psci::error::PsciError;
use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::device::vm_exit::DeviceVmExitHandler;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle mmio, err: {0}")]
    MmioErr(#[from] crate::device::Error),

    #[error("Failed to rw vcpu, err: {0}")]
    VcpuError(String),

    #[error("Failed to handle smc, err: {0}")]
    PsciError(#[from] PsciError),
}

#[derive(Debug)]
pub enum VmExitReason {
    Unknown,
    Wf,
    MMIORead {
        gpa: u64,
        srt: CoreRegister,
        len: usize,
    },
    MMIOWrite {
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
    vcpu: &dyn AArch64Vcpu,
    exit_reason: VmExitReason,
    psci: &dyn Psci,
    device: &dyn DeviceVmExitHandler,
) -> Result<HandleVmExitResult, Error> {
    trace!(?exit_reason);

    match exit_reason {
        VmExitReason::Unknown => Ok(HandleVmExitResult::Continue),
        VmExitReason::Wf => Ok(HandleVmExitResult::NextInstruction),
        VmExitReason::MMIORead { gpa, srt, len } => {
            let mut buf = [0; 8];
            device.mmio_read(gpa, len, &mut buf[0..len])?;
            vcpu.set_core_reg(srt, u64::from_le_bytes(buf))
                .map_err(|err| Error::VcpuError(err.to_string()))?;
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::MMIOWrite { gpa, buf, len } => {
            device.mmio_write(gpa, len, &buf)?;
            Ok(HandleVmExitResult::NextInstruction)
        }
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
            psci.call(vcpu)?;

            Ok(HandleVmExitResult::NextInstruction)
        }
    }
}
