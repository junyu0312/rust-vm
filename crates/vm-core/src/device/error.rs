use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Failed to save device snapshot, error: {0}")]
    Save(Box<dyn std::error::Error + Send + Sync>),
}
