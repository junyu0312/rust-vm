use thiserror::Error;
use vm_core::cpu::error::CpuError;

use crate::service::gdbstub::command::GdbStubCommandRequest;
use crate::service::monitor::command::MonitorCommandRequest;

pub(crate) mod gdbstub;
pub(crate) mod monitor;

pub enum VmmCommand {
    GdbCommand(GdbStubCommandRequest),
    MonitorCommand(MonitorCommandRequest),
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("vCPU with ID {vcpu_id} does not exist")]
    VcpuNotExists { vcpu_id: usize },

    #[error("Vm error: {0}")]
    VmError(#[from] crate::error::Error),

    #[error("Cpu error: {0}")]
    CpuError(#[from] CpuError),

    #[error("Failed to send response to command request")]
    FailedToSendResponse,

    #[error("Invalid Command")]
    InvalidCommand,
}
