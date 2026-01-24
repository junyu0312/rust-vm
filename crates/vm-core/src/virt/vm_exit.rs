use tracing::debug;

use crate::device::IoAddressSpace;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::virt::hvp::vcpu::HvpVcpu;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to handle mmio, err: {0}")]
    MmioErr(String),
}

#[derive(Debug)]
pub enum Rw {
    Write(u64), // Data
    Read(u64),  // Register id
}

#[derive(Debug)]
pub enum VmExitReason {
    Unknown,
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
}

pub enum HandleVmExitResult {
    Continue,
}

pub fn handle_vm_exit(
    vcpu: &HvpVcpu,
    exit_reason: VmExitReason,
    device: &mut IoAddressSpace,
) -> Result<HandleVmExitResult, Error> {
    debug!(?exit_reason);

    match exit_reason {
        VmExitReason::Unknown => Ok(HandleVmExitResult::Continue),
        VmExitReason::MMIORead { gpa, srt, len } => {
            let mut buf = [0; 8];
            device
                .mmio_read(gpa, len, &mut buf[0..len])
                .map_err(|err| Error::MmioErr(err.to_string()))?;
            vcpu.set_core_reg(srt, u64::from_le_bytes(buf))
                .map_err(|err| Error::MmioErr(err.to_string()))?;
            Ok(HandleVmExitResult::Continue)
        }
        VmExitReason::MMIOWrite { gpa, buf, len } => {
            device
                .mmio_write(gpa, len, &buf)
                .map_err(|err| Error::MmioErr(err.to_string()))?;
            Ok(HandleVmExitResult::Continue)
        }
    }
}
