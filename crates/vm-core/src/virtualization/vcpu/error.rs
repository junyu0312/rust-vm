use thiserror::Error;

use crate::cpu::vm_exit::VmExitHandlerError;

#[derive(Error, Debug)]
pub enum VcpuError {
    #[error("Vcpu command channel disconnected")]
    VcpuCommandDisconnected,

    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),

    #[cfg(feature = "kvm")]
    #[error("{0}")]
    KvmError(#[from] kvm_ioctls::Error),

    #[error("{0}")]
    VmExitHandlerError(#[from] VmExitHandlerError),

    #[error("Guest error: {0}")]
    GuestError(String),

    #[error("Failed to translate gpa")]
    TranslateErr,
}
