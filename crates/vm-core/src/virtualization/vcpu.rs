use async_trait::async_trait;

use crate::arch::registers::ArchCoreRegisters;
use crate::arch::registers::ArchRegisters;
use crate::cpu::error::VcpuError;

pub(crate) mod command;

#[async_trait]
pub trait HypervisorVcpu: Send {
    fn vcpu_id(&self) -> usize;

    async fn read_reigsters(&mut self) -> Result<ArchRegisters, VcpuError>;

    async fn write_registers(&mut self, registers: ArchRegisters) -> Result<(), VcpuError>;

    async fn read_core_registers(&mut self) -> Result<ArchCoreRegisters, VcpuError>;

    async fn write_core_registers(&mut self, registers: ArchCoreRegisters)
    -> Result<(), VcpuError>;

    async fn translate_gva_to_gpa(&self, gva: u64) -> Result<u64, VcpuError>;

    async fn resume(&mut self) -> Result<(), VcpuError>;

    async fn pause(&mut self) -> Result<(), VcpuError>;
}
