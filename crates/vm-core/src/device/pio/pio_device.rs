use crate::device::Device;
use crate::device::address_space::Range;

pub type PortRange = Range<u16>;

pub trait PioDevice: Device {
    fn ports(&self) -> Vec<PortRange>;

    fn io_in(&mut self, port: u16, data: &mut [u8]);

    fn io_out(&mut self, port: u16, data: &[u8]);
}
