use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorServerError {
    #[error("Failed to send request")]
    FailedToSendRequest,

    #[error("Failed to receive response")]
    FailedToReceiveResponse,
}
