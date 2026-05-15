use std::path::PathBuf;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::service::monitor::error::MonitorServerError;
use crate::vmm::handler::VmmCommand;

#[derive(Debug, PartialEq, Eq)]
pub enum MonitorCommand {
    Pause,
    Resume,
    Save(PathBuf),
}

pub struct MonitorCommandRequest {
    pub command: MonitorCommand,
    pub response: oneshot::Sender<MonitorCommandResponse>,
}

#[derive(Debug)]
pub enum MonitorCommandResponse {
    Ok,
    Err(Box<dyn std::error::Error + Send + Sync>),
}

impl MonitorCommand {
    pub async fn send_and_then_wait(
        self,
        tx: &mpsc::Sender<VmmCommand>,
    ) -> Result<MonitorCommandResponse, MonitorServerError> {
        let (response_tx, response_rx) = oneshot::channel();

        let request = VmmCommand::MonitorCommand(MonitorCommandRequest {
            command: self,
            response: response_tx,
        });

        tx.send(request).await?;

        let response = response_rx.await?;

        Ok(response)
    }
}
