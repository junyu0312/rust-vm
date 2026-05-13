use tracing::trace;

use crate::error::Error;
use crate::service::gdbstub::command::GdbStubCommand;
use crate::service::gdbstub::command::GdbStubCommandResponse;
use crate::vmm::Vmm;

impl Vmm {
    pub async fn handle_gdbstub_command(
        &mut self,
        cmd: GdbStubCommand,
    ) -> Result<GdbStubCommandResponse, Error> {
        match cmd {
            GdbStubCommand::ReadRegisters { vcpu_id } => {
                trace!(vcpu_id, "ReadRegisters");

                let registers = self.try_get_vm()?.read_core_registers(vcpu_id).await?;

                Ok(GdbStubCommandResponse::ReadRegisters {
                    registers: Box::new(registers.into()),
                })
            }
            GdbStubCommand::WriteRegisters { vcpu_id, registers } => {
                trace!(vcpu_id, "WriteRegisters");

                self.try_get_vm()?
                    .write_core_registers(vcpu_id, (*registers).into())
                    .await?;

                Ok(GdbStubCommandResponse::WriteRegisters)
            }
            GdbStubCommand::ReadAddrs { gva, len, vcpu_id } => {
                trace!(gva, len, vcpu_id, "ReadAddrs");

                let buf = self.try_get_vm()?.read_addrs(gva, len, vcpu_id).await?;

                Ok(GdbStubCommandResponse::ReadAddrs { buf })
            }
            GdbStubCommand::WriteAddrs { gva, data, vcpu_id } => {
                trace!(gva, len = data.len(), vcpu_id, "WriteAddrs");

                self.try_get_vm_mut()?
                    .write_addrs(gva, &data, vcpu_id)
                    .await?;

                Ok(GdbStubCommandResponse::WriteAddrs)
            }
            GdbStubCommand::ListActiveThreads => {
                trace!("ListActiveThreads");

                let vcpu = self.try_get_vm()?.get_active_vcpus().await;

                Ok(GdbStubCommandResponse::ListActiveThreads(vcpu))
            }
            GdbStubCommand::Resume => {
                trace!("Resume");

                self.try_get_vm_mut()?.resume().await?;

                Ok(GdbStubCommandResponse::Resume)
            }
        }
    }
}
