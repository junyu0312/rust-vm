use tracing::error;

use crate::service::monitor::command::MonitorCommand;
use crate::service::monitor::command::MonitorCommandResponse;
use crate::vmm::Vmm;

impl Vmm {
    pub async fn handle_monitor_client_command(
        &mut self,
        cmd: MonitorCommand,
    ) -> MonitorCommandResponse {
        async {
            match cmd {
                MonitorCommand::Pause => {
                    self.pause().await?;

                    Ok(MonitorCommandResponse::Ok)
                }
                MonitorCommand::Resume => todo!(),
                MonitorCommand::Save(path) => {
                    self.save(path).await?;

                    Ok(MonitorCommandResponse::Ok)
                }
            }
        }
        .await
        .unwrap_or_else(|err| {
            error!(?err, "Failed to handle monitor command");

            MonitorCommandResponse::Err(err)
        })

        /*
                let Some(command) = tokens.next() else {
                    return Err(CommandError::InvalidCommand);
                };

                let subcommands: Vec<&str> = tokens.collect();

                let vm = self.try_get_vm()?;
                let Some(handler) = vm.monitor_handlers().get(command) else {
                    return Err(CommandError::InvalidCommand);
                };

                match handler.handle_command(&subcommands).await {
                    Ok(resp) => Ok(MonitorCommandResponse(resp)),
                    Err(err) => Ok(MonitorCommandResponse(err.to_string())),
                }
        */
    }
}
