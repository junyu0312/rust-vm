use std::sync::Arc;
use std::sync::Mutex;

use tracing::trace;

use crate::arch::aarch64::firmware::psci::Psci;
use crate::arch::aarch64::vcpu::AArch64Vcpu;
use crate::arch::aarch64::vcpu::reg::CoreRegister;
use crate::arch::aarch64::vcpu::reg::SysRegister;
use crate::arch::aarch64::vm_exit::smc::handle_smc;
use crate::device::vm_exit::DeviceVmExitHandler;

pub mod smc;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle mmio, err: {0}")]
    MmioErr(String),
    #[error("Failed to handle smc, err: {0}")]
    SmcErr(String),
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
    psci: Arc<Mutex<dyn Psci>>,
    device: Arc<dyn DeviceVmExitHandler>,
) -> Result<HandleVmExitResult, Error> {
    trace!(?exit_reason);

    match exit_reason {
        VmExitReason::Unknown => Ok(HandleVmExitResult::Continue),
        VmExitReason::Wf => Ok(HandleVmExitResult::NextInstruction),
        VmExitReason::MMIORead { gpa, srt, len } => {
            let mut buf = [0; 8];
            device
                .mmio_read(gpa, len, &mut buf[0..len])
                .map_err(|err| Error::MmioErr(err.to_string()))?;
            vcpu.set_core_reg(srt, u64::from_le_bytes(buf))
                .map_err(|err| Error::MmioErr(err.to_string()))?;
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::MMIOWrite { gpa, buf, len } => {
            device
                .mmio_write(gpa, len, &buf)
                .map_err(|err| Error::MmioErr(err.to_string()))?;
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::TrappedRead { .. } => Ok(HandleVmExitResult::NextInstruction),
        VmExitReason::TrappedWrite { reg, .. } => {
            match reg {
                SysRegister::OslarEl1 => (),
                SysRegister::OsdlrEl1 => (),
                _ => unimplemented!(),
            }
            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::Smc => {
            handle_smc(psci, vcpu)?;

            Ok(HandleVmExitResult::NextInstruction)
        }
    }
}
