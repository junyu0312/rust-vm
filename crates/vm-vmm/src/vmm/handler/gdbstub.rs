use tracing::trace;

use crate::service::gdbstub::command::GdbStubCommand;
use crate::service::gdbstub::command::GdbStubCommandResponse;
use crate::vmm::Vmm;
use crate::vmm::handler::CommandError;

impl Vmm {
    pub async fn handle_gdbstub_command(
        &mut self,
        cmd: GdbStubCommand,
    ) -> Result<GdbStubCommandResponse, CommandError> {
        match cmd {
            GdbStubCommand::ReadRegisters { vcpu_id } => {
                trace!(vcpu_id, "ReadRegisters");

                let vcpu_manager = self.try_get_vm()?.vcpu_manager();
                let mut vcpu_manager = vcpu_manager.lock().await;
                let vcpu = vcpu_manager
                    .get_vcpu_mut(vcpu_id)
                    .map_err(|_| CommandError::VcpuNotExists { vcpu_id })?;

                let registers = vcpu.read_core_registers().await.unwrap();

                Ok(GdbStubCommandResponse::ReadRegisters {
                    registers: Box::new(registers.into()),
                })
            }
            GdbStubCommand::WriteRegisters { vcpu_id, registers } => {
                trace!(vcpu_id, "WriteRegisters");

                let vcpu_manager = self.try_get_vm_mut()?.vcpu_manager();
                let mut vcpu_manager = vcpu_manager.lock().await;
                let vcpu = vcpu_manager
                    .get_vcpu_mut(vcpu_id)
                    .map_err(|_| CommandError::VcpuNotExists { vcpu_id })?;

                vcpu.write_core_registers((*registers).into()).await?;

                Ok(GdbStubCommandResponse::WriteRegisters)
            }
            GdbStubCommand::ReadAddrs { gva, len, vcpu_id } => {
                trace!(gva, len, vcpu_id, "ReadAddrs");

                let vm = self.try_get_vm_mut()?;
                let vcpu_manager = vm.vcpu_manager();
                let vcpu_manager = vcpu_manager.lock().await;
                let vcpu = vcpu_manager
                    .get_vcpu(vcpu_id)
                    .map_err(|_| CommandError::VcpuNotExists { vcpu_id })?;

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
                let _vcpu = vcpu_manager
                    .get_vcpu(vcpu_id)
                    .map_err(|_| CommandError::VcpuNotExists { vcpu_id })?;

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
}
