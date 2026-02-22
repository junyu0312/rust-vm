use tracing::trace;

use crate::device::vm_exit::DeviceVmExitHandler;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::{
    AArch64TrappedRegister, CoreRegister, DebugSysRegister, SpecialPurposeRegister,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle mmio, err: {0}")]
    MmioErr(String),
    #[error("Failed to handle trapped register, err: {0}")]
    TrappedRegisterErr(String),
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
        reg: AArch64TrappedRegister,
        rt: CoreRegister,
    },
    TrappedWrite {
        reg: AArch64TrappedRegister,
        data: u64,
    },
}

pub enum HandleVmExitResult {
    Continue,
    NextInstruction,
}

pub fn handle_vm_exit(
    vcpu: &dyn AArch64Vcpu,
    exit_reason: VmExitReason,
    device: &mut dyn DeviceVmExitHandler,
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
        VmExitReason::TrappedRead { reg, rt } => {
            match reg {
                AArch64TrappedRegister::DebugSys(reg) => todo!(),
                AArch64TrappedRegister::SpecialPurpose(reg) => match reg {
                    SpecialPurposeRegister::ICC_PMR_EL1 => {
                        let value = vcpu
                            .get_icc_pmr_el1()
                            .map_err(|err| Error::TrappedRegisterErr(err.to_string()))?;
                        vcpu.set_core_reg(rt, value)
                            .map_err(|err| Error::TrappedRegisterErr(err.to_string()))?;
                    }
                },
            }

            Ok(HandleVmExitResult::NextInstruction)
        }
        VmExitReason::TrappedWrite { reg, .. } => {
            match reg {
                AArch64TrappedRegister::DebugSys(reg) => match reg {
                    DebugSysRegister::OslarEl1 => todo!(),
                    DebugSysRegister::OslsrEl1 => todo!(),
                    DebugSysRegister::OsdlrEl1 => todo!(),
                },
                AArch64TrappedRegister::SpecialPurpose(reg) => match reg {
                    SpecialPurposeRegister::ICC_PMR_EL1 => todo!(),
                },
            }
            Ok(HandleVmExitResult::NextInstruction)
        }
    }
}
