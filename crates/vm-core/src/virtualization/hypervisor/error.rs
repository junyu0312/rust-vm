use thiserror::Error;

#[derive(Error, Debug)]
pub enum HypervisorError {
    #[error("Failed to create vm: {0}")]
    CreateVm(String),

    #[error("Kvm error: {0}")]
    Kvm(#[from] kvm_ioctls::Error),
}
