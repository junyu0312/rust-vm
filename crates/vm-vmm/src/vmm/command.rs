use thiserror::Error;
use tracing::error;
use tracing::trace;
use vm_core::cpu::error::VcpuError;

use crate::service::gdbstub::command::GdbStubCommand;
use crate::service::gdbstub::command::GdbStubCommandError;
use crate::service::gdbstub::command::GdbStubCommandRequest;
use crate::service::gdbstub::command::GdbStubCommandResponse;
use crate::vm::Vm;
use crate::vmm::Vmm;

pub enum VmmCommand {
    GdbCommand(GdbStubCommandRequest),
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("VM instance does not exist")]
    VmNotExists,

    #[error("vCPU with ID {vcpu_id} does not exist")]
    VcpuNotExists { vcpu_id: usize },

    #[error("Vcpu error: {0}")]
    VcpuError(#[from] VcpuError),

    #[error("Vm error: {0}")]
    VmError(#[from] crate::error::Error),

    #[error("Failed to send response to command request")]
    FailedToSendResponse,
}

impl Vmm {
    fn try_get_vm(&self) -> Result<&Vm, CommandError> {
        self.vm.as_ref().ok_or(CommandError::VmNotExists)
    }

    fn try_get_vm_mut(&mut self) -> Result<&mut Vm, CommandError> {
        self.vm.as_mut().ok_or(CommandError::VmNotExists)
    }

    async fn handle_gdbstub_command(
        &mut self,
        cmd: GdbStubCommand,
    ) -> Result<GdbStubCommandResponse, CommandError> {
        match cmd {
            GdbStubCommand::ReadRegisters { vcpu_id } => {
                trace!(vcpu_id, "ReadRegisters");

                let vcpu_manager = self.try_get_vm()?.vcpu_manager();
                let mut vcpu_manager = vcpu_manager.lock().await;
                let vcpu = vcpu_manager.get_vcpu_mut(vcpu_id)?;

                let registers = vcpu.read_core_registers().await.unwrap();

                Ok(GdbStubCommandResponse::ReadRegisters {
                    registers: Box::new(registers.into()),
                })
            }
            GdbStubCommand::WriteRegisters { vcpu_id, registers } => {
                trace!(vcpu_id, "WriteRegisters");

                let vcpu_manager = self.try_get_vm_mut()?.vcpu_manager();
                let mut vcpu_manager = vcpu_manager.lock().await;
                let vcpu = vcpu_manager.get_vcpu_mut(vcpu_id)?;

                vcpu.write_core_registers((*registers).into()).await?;

                Ok(GdbStubCommandResponse::WriteRegisters)
            }
            GdbStubCommand::ReadAddrs { gva, len, vcpu_id } => {
                trace!(gva, len, vcpu_id, "ReadAddrs");

                let vm = self.try_get_vm_mut()?;
                let vcpu_manager = vm.vcpu_manager();
                let vcpu_manager = vcpu_manager.lock().await;
                let vcpu = vcpu_manager.get_vcpu(vcpu_id)?;

                let mut len = len;
                let mut buf = Vec::with_capacity(len);
                while len > 0 {
                    let Some(gpa) = vcpu.translate_gva_to_gpa(gva).await? else {
                        return Ok(GdbStubCommandResponse::Err);
                    };

                    let hva = vm.memory_address_space().gpa_to_hva(gpa).unwrap();
                    buf.push(unsafe { *hva });
                    // TODO: Opt
                    len -= 1;
                }

                Ok(GdbStubCommandResponse::ReadAddrs { buf })
            }
            GdbStubCommand::WriteAddrs { gva, data, vcpu_id } => {
                trace!(gva, len = data.len(), vcpu_id, "WriteAddrs");

                let vm = self.try_get_vm_mut()?;
                let vcpu_manager = vm.vcpu_manager();
                let vcpu_manager = vcpu_manager.lock().await;
                let _vcpu = vcpu_manager.get_vcpu(vcpu_id)?;

                let _buf = todo!();

                // Ok(GdbStubCommandResponse::WriteAddrs)
            }
            GdbStubCommand::ListActiveThreads => {
                trace!("ListActiveThreads");

                let vm = self.try_get_vm()?;
                let vcpu = vm.vcpu_manager().lock().await.get_active_vcpus();
                Ok(GdbStubCommandResponse::ListActiveThreads(vcpu))
            }
            GdbStubCommand::Resume => {
                trace!("Resume");

                let vm = self.try_get_vm_mut()?;
                vm.resume().await?;
                Ok(GdbStubCommandResponse::Resume)
            }
        }
    }

    async fn handle_command(&mut self, command: VmmCommand) -> Result<(), CommandError> {
        match command {
            VmmCommand::GdbCommand(cmd) => {
                let r = self
                    .handle_gdbstub_command(cmd.command)
                    .await
                    .inspect_err(|err| {
                        error!(?err, "Failed to handle GDB stub command");
                    })
                    .map_err(|_| GdbStubCommandError::Err);

                cmd.response
                    .send(r)
                    .map_err(|_| CommandError::FailedToSendResponse)?;
            }
        }

        Ok(())
    }

    pub async fn run_monitor(&mut self) -> Result<(), CommandError> {
        while let Some(command) = self.command_rx.recv().await {
            self.handle_command(command).await?;
        }

        Ok(())
    }
}
