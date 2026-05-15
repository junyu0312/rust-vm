use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::oneshot::error::RecvError;

use crate::vmm::handler::VmmCommand;

#[derive(Error, Debug)]
pub enum MonitorServerError {
    #[error("Failed to send request")]
    SendRequest(#[from] SendError<VmmCommand>),

    #[error("Failed to receive response")]
    ReceiveResponse(#[from] RecvError),

    #[error("Failed to parse command, {0}")]
    ParseCommand(winnow::error::ContextError),
}
