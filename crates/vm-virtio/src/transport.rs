use crate::types::device::Subsystem;
use crate::types::status::Status;

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

pub trait VirtIo {
    type Subsystem: Subsystem;

    const NAME: &str;
    const VIRT_QUEUES: u32;

    fn reset(&mut self);

    fn read_device_features(&self) -> u32;

    fn write_device_feature_sel(&mut self, sel: u32);

    fn write_driver_features(&mut self, feat: u32);

    fn write_driver_feature_sel(&mut self, sel: u32);

    fn write_queue_sel(&mut self, sel: u32);

    fn read_queue_size_max(&self) -> u32;

    fn write_queue_size(&mut self, size: u16);

    fn read_queue_ready(&self) -> bool;

    fn write_queue_ready(&mut self, queue_ready: bool);

    fn write_queue_notify(&mut self, val: u32);

    fn read_interrupt_status(&self) -> u32;

    fn write_interrupt_ack(&mut self, val: u32);

    fn read_status(&self) -> Status;

    fn write_status_non_zero(&mut self, val: Status);

    fn write_status(&mut self, val: u8) {
        if val == 0 {
            self.reset()
        } else {
            self.write_status_non_zero(Status::from_bits_truncate(val))
        }
    }

    fn write_queue_desc_low(&mut self, addr: u32);

    fn write_queue_desc_high(&mut self, addr: u32);

    fn write_queue_driver_low(&mut self, addr: u32);

    fn write_queue_driver_high(&mut self, addr: u32);

    fn write_queue_device_low(&mut self, addr: u32);

    fn write_queue_device_high(&mut self, addr: u32);

    fn read_config_generation(&self) -> u32;
}
