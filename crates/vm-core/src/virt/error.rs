#[derive(Debug, thiserror::Error)]
pub enum VirtError {
    #[error("Failed to initialize virt, reason: {0}")]
    FailedInitialize(String),
}
