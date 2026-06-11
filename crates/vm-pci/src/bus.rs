use std::collections::BTreeMap;

use crate::types::device::PciDevice;

#[derive(Default)]
pub struct PciBus {
    devices: BTreeMap<u8, Box<dyn PciDevice>>,
}

impl PciBus {
    pub fn get_device(&self, device_number: u8) -> Option<&dyn PciDevice> {
        self.devices.get(&device_number).map(|dev| dev.as_ref())
    }

    pub fn devices(&self) -> impl Iterator<Item = (&u8, &dyn PciDevice)> {
        self.devices.iter().map(|(id, dev)| (id, dev.as_ref()))
    }

    pub fn devices_mut(&mut self) -> impl Iterator<Item = (&u8, &mut (dyn PciDevice + 'static))> {
        self.devices.iter_mut().map(|(id, dev)| (id, dev.as_mut()))
    }

    pub fn register_device(&mut self, device_id: u8, device: Box<dyn PciDevice>) {
        let old_dev = self.devices.insert(device_id, device);

        assert!(old_dev.is_none());
    }
}
