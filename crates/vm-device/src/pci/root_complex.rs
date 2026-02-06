use std::collections::HashMap;

use tracing::debug;

use crate::pci::bus::PciBus;
use crate::pci::device::PciDevice;
use crate::pci::host_bridge::PciHostBridge;

pub mod mmio;
pub mod pio;

#[derive(Default)]
struct PciRootComplex {
    bus: HashMap<u8, PciBus>,
    host_bridge: Box<PciHostBridge>,
}

impl PciRootComplex {
    fn get_device(&self, bus_number: u8, device_number: u8) -> Option<&dyn PciDevice> {
        if bus_number == 0 && device_number == 0 {
            return Some(self.host_bridge.as_ref() as &dyn PciDevice);
        }

        self.bus
            .get(&bus_number)
            .and_then(|bus| bus.get_device(device_number))
    }

    fn get_device_mut(&mut self, bus_number: u8, device_number: u8) -> Option<&mut dyn PciDevice> {
        debug!(bus_number, device_number);

        if bus_number == 0 && device_number == 0 {
            return Some(self.host_bridge.as_mut() as &mut dyn PciDevice);
        }

        self.bus
            .get_mut(&bus_number)
            .and_then(|bus| bus.get_device_mut(device_number))
    }

    fn handle_ecam_read(&self, bus: u8, device: u8, func: u8, offset: u16, data: &mut [u8]) {
        let Some(device) = self.get_device(bus, device) else {
            // When a configuration access attempts to select a device that does not exist,
            // the host bridge will complete the access without error, dropping all data on
            // writes and returning all ones on reads.
            data.fill(0xff);
            return;
        };

        let func = device.get_func(func);
        let configuration_space = func.get_configuration_space();
        configuration_space.read(offset, data);
    }

    fn handle_ecam_write(&mut self, _bus: u8, _devicee: u8, _func: u8, _offset: u16, _data: &[u8]) {
        todo!()
    }
}
