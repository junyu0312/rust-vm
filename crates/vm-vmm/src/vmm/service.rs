use tracing::error;

use crate::vmm::Vmm;
use crate::vmm::handler::CommandError;
use crate::vmm::handler::VmmCommand;

mod monitor;

impl Vmm {
    pub async fn run_monitor(&mut self) {
        self.listen_for_monitor_client();

        while let Some(command) = self.command_rx.recv().await {
            if let Err(err) = self.handle_command(command).await {
                error!(?err, "Failed to handle command");
            }
        }
    }

    async fn handle_command(&mut self, command: VmmCommand) -> Result<(), CommandError> {
        match command {
            VmmCommand::GdbCommand(cmd) => {
                let r = self.handle_gdbstub_command(cmd.command).await?;

                cmd.response
                    .send(r)
                    .map_err(|_| CommandError::FailedToSendResponse)?;
            }
            VmmCommand::MonitorCommand(cmd) => {
                let r = self.handle_monitor_client_command(cmd.command).await?;

                cmd.response
                    .send(r)
                    .map_err(|_| CommandError::FailedToSendResponse)?;
            }
        }

        Ok(())
    }
}
