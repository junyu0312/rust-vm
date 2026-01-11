use std::collections::HashMap;

use crate::pci::device::PciDevice;

pub struct PciBus {
    devices: HashMap<u8, Box<dyn PciDevice>>,
}

impl PciBus {
    pub fn get_device(&self, device_number: u8) -> Option<&dyn PciDevice> {
        self.devices.get(&device_number).map(|d| d.as_ref() as _)
    }

    pub fn get_device_mut(&mut self, device_number: u8) -> Option<&mut dyn PciDevice> {
        self.devices
            .get_mut(&device_number)
            .map(|d| d.as_mut() as _)
    }
}
