use crate::virtio::transport::Result;

pub mod virtio_input;

pub trait Subsystem {
    type DeviceConfiguration;

    const DEVICE_ID: u32;

    fn read_device_configuration(&self, offset: usize, len: usize, data: &mut [u8]) -> Result<()>;

    fn write_device_configuration(&mut self, offset: usize, len: usize, data: &[u8]) -> Result<()>;
}
