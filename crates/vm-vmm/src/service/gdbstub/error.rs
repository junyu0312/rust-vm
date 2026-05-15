use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::oneshot::error::RecvError;

use crate::vmm::handler::VmmCommand;

#[derive(Error, Debug)]
pub enum VmGdbStubError {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to send command to VMM")]
    FailedToSendCommand(#[from] SendError<VmmCommand>),

    #[error("Failed to receive response from VMM")]
    FailedToReceiveCommandResponse(#[from] RecvError),

    #[error("Failed to list active threads")]
    ListActiveThreadsFailed,

    #[error("Failed to resume")]
    ResumeFailed,

    #[error("invalid thread ID")]
    InvalidTid,
}
