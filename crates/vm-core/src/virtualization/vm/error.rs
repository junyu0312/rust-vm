use thiserror::Error;

use crate::cpu::error::CpuError;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Vcpu {0} is not exists")]
    VcpuNotCreated(usize),

    #[error("Failed to create vcpu: {0}")]
    CreateVcpuError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to create irq_chip: {0}")]
    CreateIrqChipError(String),

    #[error("Failed to set_user_memory_region: {0}")]
    SetUserMemoryRegionError(String),

    #[cfg(feature = "hvp")]
    #[error("Applevisor error: {0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),

    #[error("Cpu error: {0}")]
    CpuError(#[from] CpuError),
}
