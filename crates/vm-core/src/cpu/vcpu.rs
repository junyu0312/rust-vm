use tokio::sync::mpsc::WeakSender;

use crate::arch::registers::ArchCoreRegisters;
use crate::arch::registers::ArchRegisters;
use crate::cpu::error::VcpuError;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vcpu::command::VcpuCommand;
use crate::virtualization::vcpu::command::VcpuCommandRequest;
use crate::virtualization::vcpu::command::VcpuCommandResponse;

pub struct Vcpu {
    command_tx: WeakSender<VcpuCommandRequest>,
    vcpu_instance: Box<dyn HypervisorVcpu>,
    booted: bool,
}

impl Vcpu {
    pub fn new(vcpu_instance: Box<dyn HypervisorVcpu>) -> Self {
        Vcpu {
            command_tx: vcpu_instance.command_tx(),
            vcpu_instance,
            booted: false,
        }
    }

    pub fn vcpu_id(&self) -> usize {
        self.vcpu_instance.vcpu_id()
    }

    pub async fn boot_vcpu(
        &mut self,
        pc: u64,
        dtb_or_context_id: u64,
        stop_on_boot: bool,
    ) -> Result<(), VcpuError> {
        #[cfg(target_arch = "aarch64")]
        {
            use crate::arch::registers::aarch64::AArch64Registers;

            let register = self.read_registers().await?;
            let registers =
                AArch64Registers::boot_registers(self.vcpu_id(), dtb_or_context_id, pc, register);
            self.write_registers(registers).await?;
        }

        #[cfg(target_arch = "x86_64")]
        {
            use std::hint::black_box;

            black_box((pc, dtb_or_context_id));
        }

        self.booted = true;

        if !stop_on_boot {
            self.resume().await?;
        }

        Ok(())
    }

    pub async fn read_registers(&mut self) -> Result<ArchRegisters, VcpuError> {
        match self
            .send_command_and_then_wait(VcpuCommand::ReadRegisters)
            .await?
        {
            VcpuCommandResponse::Registers(regs) => Ok(*regs),
            _ => unreachable!(),
        }
    }

    pub async fn read_core_registers(&mut self) -> Result<ArchCoreRegisters, VcpuError> {
        match self
            .send_command_and_then_wait(VcpuCommand::ReadCoreRegisters)
            .await?
        {
            VcpuCommandResponse::CoreRegisters(regs) => Ok(*regs),
            _ => unreachable!(),
        }
    }

    pub async fn write_core_registers(
        &mut self,
        registers: ArchCoreRegisters,
    ) -> Result<(), VcpuError> {
        match self
            .send_command_and_then_wait(VcpuCommand::WriteCoreRegisters(registers))
            .await?
        {
            VcpuCommandResponse::Empty => Ok(()),
            _ => unreachable!(),
        }
    }

    pub async fn write_registers(&mut self, registers: ArchRegisters) -> Result<(), VcpuError> {
        match self
            .send_command_and_then_wait(VcpuCommand::WriteRegisters(registers))
            .await?
        {
            VcpuCommandResponse::Empty => Ok(()),
            _ => unreachable!(),
        }
    }

    pub async fn translate_gva_to_gpa(&self, gva: u64) -> Result<Option<u64>, VcpuError> {
        match self
            .send_command_and_then_wait(VcpuCommand::TranslateGvaToGpa(gva))
            .await?
        {
            VcpuCommandResponse::TranslateGvaToGpa(gpa) => Ok(gpa),
            _ => unreachable!(),
        }
    }

    pub async fn resume(&mut self) -> Result<(), VcpuError> {
        if !self.booted {
            return Ok(());
        }

        match self.send_command_and_then_wait(VcpuCommand::Resume).await? {
            VcpuCommandResponse::Empty => Ok(()),
            _ => unreachable!(),
        }
    }

    pub async fn pause(&mut self) -> Result<(), VcpuError> {
        if !self.booted {
            return Ok(());
        }

        match self.send_command_and_then_wait(VcpuCommand::Pause).await? {
            VcpuCommandResponse::Empty => Ok(()),
            _ => unreachable!(),
        }
    }

    async fn send_command_and_then_wait(
        &self,
        command: VcpuCommand,
    ) -> Result<VcpuCommandResponse, VcpuError> {
        let (req, rx) = VcpuCommandRequest::new(command);

        self.command_tx
            .upgrade()
            .ok_or(VcpuError::VcpuCommandDisconnected)?
            .send(req)
            .await
            .map_err(|_| VcpuError::VcpuCommandDisconnected)?;

        rx.await.map_err(|_| VcpuError::VcpuCommandDisconnected)
    }
}
