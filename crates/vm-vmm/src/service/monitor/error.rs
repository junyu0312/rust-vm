use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorServerError {
    #[error("Failed to send request")]
    SendRequest,

    #[error("Failed to receive response")]
    ReceiveResponse,

    #[error("Failed to parse command, {0}")]
    ParseCommand(winnow::error::ContextError),
}
