use thiserror::Error;

use crate::cpu::vm_exit::VmExitHandlerError;

#[derive(Error, Debug)]
pub enum VcpuError {
    #[error("Failed to create vcpu: {0}")]
    FailedToCreateVcpu(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("vcpu {0} not created")]
    VcpuNotCreated(usize),

    #[error("Vcpu command channel disconnected")]
    VcpuCommandDisconnected,

    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),

    #[cfg(feature = "kvm")]
    #[error("{0}")]
    KvmError(#[from] kvm_ioctls::Error),

    #[error("{0}")]
    GuestError(String),

    #[error("{0}")]
    VmExitHandlerErr(#[from] VmExitHandlerError),

    #[error("Failed to translate va to pa")]
    TranslateErr,
}
