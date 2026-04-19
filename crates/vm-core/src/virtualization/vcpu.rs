use async_trait::async_trait;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::register::AArch64Registers as ArchRegisters;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::register::X86_64Registers as ArchRegisters;
use crate::cpu::error::VcpuError;

pub(crate) mod command;

#[async_trait]
pub trait HypervisorVcpu: Send {
    async fn read_reigsters(&mut self) -> Result<ArchRegisters, VcpuError>;

    async fn write_registers(&mut self, registers: ArchRegisters) -> Result<(), VcpuError>;

    async fn resume(&mut self) -> Result<(), VcpuError>;

    async fn pause(&mut self) -> Result<(), VcpuError>;
}
