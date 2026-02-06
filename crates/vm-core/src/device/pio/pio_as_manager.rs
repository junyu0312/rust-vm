use crate::device::Error;
use crate::device::Result;
use crate::device::address_space::AddressSpace;
use crate::device::pio::pio_device::PioDevice;
use crate::device::pio::pio_device::PortRange;

#[derive(Default)]
pub struct PioAddressSpaceManager {
    device: Vec<Box<dyn PioDevice>>,
    address_space: AddressSpace<u16>,
}

impl PioAddressSpaceManager {
    pub fn register(&mut self, device: Box<dyn PioDevice>) -> Result<()> {
        for range in device.ports() {
            if self.address_space.is_overlap(range.start, range.len) {
                return Err(Error::InvalidRange);
            }
        }

        let idx = self.device.len();

        for range in device.ports() {
            self.address_space.try_insert(range, idx)?;
        }

        self.device.push(device);

        Ok(())
    }

    pub fn get_device_by_port(&mut self, port: u16) -> Option<(&mut dyn PioDevice)> {
        let (_, idx) = self.address_space.try_get_value_by_key(port)?;

        Some(self.device[idx].as_mut())
    }
}
