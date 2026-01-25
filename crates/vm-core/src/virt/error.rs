#[derive(Debug, thiserror::Error)]
pub enum VirtError {
    #[error("Failed to initialize virt, reason: {0}")]
    FailedInitialize(String),
    #[error("Failed to setup interrupt controller, reason: {0}")]
    InterruptControllerFailed(String),
}
