use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceSnapshotError {
    #[error("save device snapshot io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to serde: {0}")]
    Serde(String),

    #[error("failed to deserde: {0}")]
    Deserde(String),

    #[error("device {0} does not support snapshot")]
    DeviceNotSupportSnapshot(String),
}
