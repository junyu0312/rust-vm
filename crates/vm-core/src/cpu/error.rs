use thiserror::Error;

use crate::cpu::vm_exit::VmExitHandlerError;

#[derive(Error, Debug)]
pub enum VcpuError {
    #[error("vcpu {0} not created")]
    VcpuNotCreated(usize),

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
}
