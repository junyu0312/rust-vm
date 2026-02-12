use std::collections::BTreeMap;

use crate::device::pci_device::PciDevice;

#[derive(Default)]
pub struct PciBus {
    devices: BTreeMap<u8, PciDevice>,
}

impl PciBus {
    pub fn get_device(&self, device_number: u8) -> Option<&PciDevice> {
        self.devices.get(&device_number)
    }

    pub fn get_device_mut(&mut self, device_number: u8) -> Option<&mut PciDevice> {
        self.devices.get_mut(&device_number)
    }

    pub fn register_device(&mut self, device_id: u8, device: PciDevice) {
        let old_dev = self.devices.insert(device_id, device);

        assert!(old_dev.is_none());
    }
}
