use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::device::pio::PioDevice;

#[derive(Default)]
pub struct IoAddressSpace {
    devices: BTreeMap<usize, Box<dyn PioDevice>>,
    port_to_device: HashMap<u16, usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("port {0:#x} is already registered")]
    PortIsAlreadyRegistered(u16),
    #[error("no device found for port {0:#x}")]
    NoDeviceForPort(u16),
}

impl IoAddressSpace {
    pub fn register(&mut self, device: Box<dyn PioDevice>) -> Result<(), Error> {
        let id = self.devices.len();
        for &port in device.ports() {
            if self.port_to_device.contains_key(&port) {
                return Err(Error::PortIsAlreadyRegistered(port));
            }
        }

        for &port in device.ports() {
            self.port_to_device.insert(port, id);
        }
        self.devices.insert(id, device);

        Ok(())
    }

    fn get_mut_device_by_port(&mut self, port: u16) -> Option<&mut Box<dyn PioDevice>> {
        if let Some(device_index) = self.port_to_device.get(&port) {
            return self.devices.get_mut(device_index);
        }
        None
    }

    pub fn io_in(&mut self, port: u16, data: &mut [u8]) -> Result<(), Error> {
        let Some(device) = self.get_mut_device_by_port(port) else {
            return Err(Error::NoDeviceForPort(port));
        };

        device.io_in(port, data);

        Ok(())
    }

    pub fn io_out(&mut self, port: u16, data: &[u8]) -> Result<(), Error> {
        let Some(device) = self.get_mut_device_by_port(port) else {
            return Err(Error::NoDeviceForPort(port));
        };

        device.io_out(port, data);

        Ok(())
    }
}
