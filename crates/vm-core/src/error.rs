#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to initialize virt, reason: {0}")]
    FailedInitialize(String),
    #[error("Failed to setup interrupt controller, reason: {0}")]
    InterruptControllerFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;
