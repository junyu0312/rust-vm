use std::io::Read;
use std::io::Write;

use anyhow::anyhow;
use futures::executor::block_on;
use vm_snapshot::ops::Snapshotable;

use crate::cpu::vcpu::Vcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandResponse;
use crate::virtualization::vcpu::error::VcpuError;

impl Snapshotable for Vcpu {
    type Error = VcpuError;

    fn save(&self, writer: &mut dyn Write) -> Result<(), Self::Error> {
        writer
            .write_all(&((self.booted as u64).to_le_bytes()))
            .map_err(|err| VcpuError::Save(Box::new(err)))?;

        let state = block_on(self.send_command_and_then_wait(VcpuCommand::ReadRegisters))
            .map_err(|err| VcpuError::Save(Box::new(err)))?;

        match state {
            VcpuCommandResponse::Registers(regs) => {
                let registers_bytes =
                    serde_json::to_vec(&regs).map_err(|err| VcpuError::Save(Box::new(err)))?;
                writer
                    .write_all(&registers_bytes)
                    .map_err(|err| VcpuError::Save(Box::new(err)))?;
            }
            _ => {
                return Err(VcpuError::Save(
                    anyhow!("Failed to save vcpu").into_boxed_dyn_error(),
                ));
            }
        }

        Ok(())
    }

    fn restore(&mut self, _reader: &mut dyn Read) -> Result<(), Self::Error> {
        todo!()
    }
}
