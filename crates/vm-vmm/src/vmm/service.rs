use tracing::error;

use crate::vmm::Vmm;
use crate::vmm::handler::VmmCommand;

mod monitor;

impl Vmm {
    pub async fn run_monitor(&mut self) {
        self.listen_for_monitor_client();

        while let Some(command) = self.command_rx.recv().await {
            self.handle_command(command).await;
        }
    }

    async fn handle_command(&mut self, command: VmmCommand) {
        match command {
            VmmCommand::GdbCommand(cmd) => {
                let response = self.handle_gdbstub_command(cmd.command).await;

                if cmd.response.send(response).is_err() {
                    error!("Failed to send gdbstub command response");
                }
            }
            VmmCommand::MonitorCommand(cmd) => {
                let response = self.handle_monitor_client_command(cmd.command).await;

                if cmd.response.send(response).is_err() {
                    error!("Failed to send monitor command response");
                }
            }
        }
    }
}
