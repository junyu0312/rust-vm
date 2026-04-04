use thiserror::Error;
use tracing::error;
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

    #[error("Failed to send response to command request")]
    FailedToSendResponse,
}

impl Vmm {
    fn try_get_vm(&self) -> Result<&Vm, CommandError> {
        self.vm.as_ref().ok_or(CommandError::VmNotExists)
    }

    fn handle_gdbstub_command(
        &mut self,
        cmd: GdbStubCommand,
    ) -> Result<GdbStubCommandResponse, CommandError> {
        match cmd {
            GdbStubCommand::ReadRegisters { vcpu_id } => {
                let vm = self.try_get_vm()?;
                let vcpu = vm
                    .vcpu_manager
                    .lock()
                    .unwrap()
                    .get_vcpu(vcpu_id)
                    .ok_or(VcpuError::VcpuNotCreated(vcpu_id))?;

                vcpu.lock().unwrap().get_registers()?;

                Ok(GdbStubCommandResponse::ReadRegisters {})
            }
            GdbStubCommand::ListActiveThreads => {
                let vm = self.try_get_vm()?;
                let vcpu = vm.vcpu_manager.lock().unwrap().get_active_vcpus();
                Ok(GdbStubCommandResponse::ListActiveThreads(vcpu))
            }
        }
    }

    fn handle_command(&mut self, command: VmmCommand) -> Result<(), CommandError> {
        match command {
            VmmCommand::GdbCommand(cmd) => {
                let r = self
                    .handle_gdbstub_command(cmd.command)
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

    pub fn run_monitor(&mut self) -> Result<(), CommandError> {
        while let Some(command) = self.command_rx.blocking_recv() {
            self.handle_command(command)?;
        }

        Ok(())
    }
}
