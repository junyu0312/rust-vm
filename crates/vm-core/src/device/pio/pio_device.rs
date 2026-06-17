use std::ops::Range;

use crate::device::error::DeviceError;

pub trait PioDevice {
    fn ports(&self) -> Vec<Range<u16>>;

    fn io_in(&self, port: u16, data: &mut [u8]) -> Result<(), DeviceError>;

    fn io_out(&self, port: u16, data: &[u8]) -> Result<(), DeviceError>;
}
