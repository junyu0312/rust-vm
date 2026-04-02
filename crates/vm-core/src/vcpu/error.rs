#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::firmware::psci::error::PsciError;
use crate::device_manager::vm_exit::DeviceError;

#[derive(Debug, thiserror::Error)]
pub enum VcpuError {
    #[error("vcpu {0} not created")]
    VcpuNotCreated(usize),

    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),

    #[cfg(feature = "kvm")]
    #[error("{0}")]
    KvmError(#[from] kvm_ioctls::Error),

    #[cfg(target_arch = "aarch64")]
    #[error("{0}")]
    PsciError(#[from] PsciError),

    #[error("{0}")]
    DeviceError(#[from] DeviceError),

    #[error("{0}")]
    GuestError(String),
}
