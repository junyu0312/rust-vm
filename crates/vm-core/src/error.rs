#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to initialize virt, reason: {0}")]
    FailedInitialize(String),
    #[error("Failed to setup interrupt controller, reason: {0}")]
    InterruptControllerFailed(String),
    #[cfg(feature = "kvm")]
    #[error("{0}")]
    KvmError(kvm_ioctls::Error),
    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(applevisor::error::HypervisorError),
    #[error("{0}")]
    Internal(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[cfg(feature = "kvm")]
impl From<kvm_ioctls::Error> for Error {
    fn from(err: kvm_ioctls::Error) -> Self {
        Error::KvmError(err)
    }
}

#[cfg(feature = "hvp")]
impl From<applevisor::error::HypervisorError> for Error {
    fn from(err: applevisor::error::HypervisorError) -> Self {
        Error::ApplevisorError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
