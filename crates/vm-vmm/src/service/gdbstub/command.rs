use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::service::gdbstub::error::VmGdbStubError;
use crate::vmm::command::VmmCommand;

pub enum GdbStubCommand {
    ReadRegisters { vcpu_id: usize },

    ListActiveThreads,
}

pub enum GdbStubCommandResponse {
    ReadRegisters,

    ListActiveThreads(usize),
}

#[derive(Error, Debug)]
pub enum GdbStubCommandError {
    #[error("Err")]
    Err,
}

pub struct GdbStubCommandRequest {
    pub command: GdbStubCommand,
    pub response: oneshot::Sender<Result<GdbStubCommandResponse, GdbStubCommandError>>,
}

impl GdbStubCommand {
    pub fn send_and_then_wait(
        self,
        tx: &mpsc::Sender<VmmCommand>,
    ) -> Result<Result<GdbStubCommandResponse, GdbStubCommandError>, VmGdbStubError> {
        let (response_tx, response_rx) = oneshot::channel();

        let request = VmmCommand::GdbCommand(GdbStubCommandRequest {
            command: self,
            response: response_tx,
        });

        if let Err(err) = tx.blocking_send(request) {
            eprintln!("Failed to send GDB stub command request: {err}");
            return Err(VmGdbStubError::FailedToSendCommand);
        }

        let response = response_rx
            .blocking_recv()
            .map_err(|_| VmGdbStubError::FailedToReceiveCommandResponse)?;

        Ok(response)
    }
}
