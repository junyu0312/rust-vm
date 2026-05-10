use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorServerError {
    #[error("Failed to send request")]
    FailedToSendRequest,

    #[error("Failed to receive response")]
    FailedToReceiveResponse,

    #[error("Failed to parse command, {0}")]
    FailedToParseCommand(winnow::error::ContextError),
}
