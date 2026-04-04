use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmGdbStubError {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to send command to VMM")]
    FailedToSendCommand,

    #[error("Failed to receive response from VMM")]
    FailedToReceiveCommandResponse,

    #[error("Failed to read registers")]
    ReadRegistersFailed,

    #[error("Failed to list active threads")]
    ListActiveThreadsFailed,

    #[error("Received invalid response from VMM")]
    InvalidResponse,

    #[error("invalid thread ID")]
    InvalidTid,
}
