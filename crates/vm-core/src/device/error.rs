use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceSnapshotError {
    #[error("save device snapshot io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("device {0} does not support snapshot")]
    DeviceNotSupportSnapshot(String),
}
