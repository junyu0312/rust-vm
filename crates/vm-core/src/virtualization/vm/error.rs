use thiserror::Error;

use crate::cpu::error::CpuError;
use crate::interrupt_manager::InterruptManagerError;
use crate::virtualization::vm::state::VmState;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Vcpu {0} is not exists")]
    VcpuNotCreated(usize),

    #[error("Failed to create vcpu: {0}")]
    CreateVcpuError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to create irq_chip: {0}")]
    CreateIrqChipError(String),

    #[error("Failed to set gsi routing")]
    SetGsiRouting(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to set_user_memory_region: {0}")]
    SetUserMemoryRegionError(String),

    #[error("Failed to create memory region")]
    MemoryRegionOverlap,

    #[cfg(feature = "hvp")]
    #[error("Applevisor error: {0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),

    #[cfg(feature = "kvm")]
    #[error("Kvm error: {0}")]
    Kvm(#[from] kvm_ioctls::Error),

    #[error("Cpu error: {0}")]
    CpuError(#[from] CpuError),

    #[error("Interrupt manager error: {0}")]
    InterruptManagerError(#[from] InterruptManagerError),

    #[error("vm state is not satisfied, current: {current:?}")]
    VmState { current: VmState },

    #[error("Failed to create listener for gdbstub")]
    GdbListenerCreation,
}
