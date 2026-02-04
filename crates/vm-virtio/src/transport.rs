// use crate::types::device::Subsystem;
// use crate::types::status::Status;

pub mod mmio;

#[derive(Debug, thiserror::Error)]
pub enum VirtIoError {
    #[error("invalid length of flag")]
    InvalidFlagLen,

    #[error("invalid write device-configuration from driver")]
    DriverWriteDeviceConfigurationInvalid,

    #[error("invalid read device-configuration from driver")]
    DriverReadDeviceConfigurationInvalid,

    #[error("try to access an unready virtqueue")]
    AccessVirtqueueNotReady,

    #[error("access invalid gpa")]
    AccessInvalidGpa,
}

pub type Result<T> = core::result::Result<T, VirtIoError>;
