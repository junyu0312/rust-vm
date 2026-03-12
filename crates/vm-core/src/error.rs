#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to initialize virt, reason: {0}")]
    FailedInitialize(String),
    #[error("Failed to setup interrupt controller, reason: {0}")]
    InterruptControllerFailed(String),
    #[cfg(feature = "kvm")]
    #[error("{0}")]
    KvmError(#[from] kvm_ioctls::Error),
    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),
    #[error("{0}")]
    MemoryError(#[from] vm_mm::error::Error),
    #[error("{0}")]
    LayoutError(#[from] crate::arch::layout::Error),
    #[error("{0}")]
    Internal(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
