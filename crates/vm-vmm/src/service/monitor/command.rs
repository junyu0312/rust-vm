use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::service::monitor::error::MonitorServerError;
use crate::vmm::handler::VmmCommand;

pub struct MonitorCommand(pub String);

pub struct MonitorCommandRequest {
    pub command: MonitorCommand,
    pub response: oneshot::Sender<MonitorCommandResponse>,
}

pub struct MonitorCommandResponse(pub String);

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

        if let Err(_err) = tx.send(request).await {
            return Err(MonitorServerError::FailedToSendRequest);
        }

        let response = response_rx
            .await
            .map_err(|_| MonitorServerError::FailedToReceiveResponse)?;

        Ok(response)
    }
}
