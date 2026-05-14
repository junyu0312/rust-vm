use crate::service::monitor::command::MonitorCommand;
use crate::service::monitor::command::MonitorCommandResponse;
use crate::vmm::Vmm;
use crate::vmm::error::VmmError;

impl Vmm {
    pub async fn handle_monitor_client_command(
        &mut self,
        cmd: MonitorCommand,
    ) -> Result<MonitorCommandResponse, VmmError> {
        match cmd {
            MonitorCommand::Pause => {
                self.pause().await?;

                Ok(MonitorCommandResponse::Ok("Paused".to_string()))
            }
            MonitorCommand::Resume => todo!(),
            MonitorCommand::Save(path) => {
                let vm = self.try_get_vm_mut()?;

                // TODO: Refine error
                vm.save(path).await.unwrap();

                Ok(MonitorCommandResponse::Ok("Saved".to_string()))
            }
        }

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
