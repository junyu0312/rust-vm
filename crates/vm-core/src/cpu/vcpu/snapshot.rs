use std::io::Read;
use std::io::Write;

use futures::executor::block_on;
use vm_snapshot::ops::Snapshotable;

use crate::cpu::vcpu::Vcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandResponse;

impl Snapshotable for Vcpu {
    fn save(&self, writer: &mut dyn Write) -> Result<(), vm_snapshot::ops::Error> {
        writer.write_all(&((self.booted as u64).to_le_bytes()))?;

        let state = block_on(self.send_command_and_then_wait(VcpuCommand::ReadRegisters)).map_err(
            |err| vm_snapshot::ops::Error::VmError(format!("Failed to read registers: {}", err)),
        )?;

        match state {
            VcpuCommandResponse::Registers(regs) => {
                let registers_bytes = serde_json::to_vec(&regs)
                    .map_err(|err| vm_snapshot::ops::Error::VmError(err.to_string()))?;
                writer.write_all(&registers_bytes)?;
            }
            _ => {
                return Err(vm_snapshot::ops::Error::VmError(
                    "Failed to save vcpu".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn restore(&mut self, _reader: &mut dyn Read) -> Result<(), vm_snapshot::ops::Error> {
        todo!()
    }
}
