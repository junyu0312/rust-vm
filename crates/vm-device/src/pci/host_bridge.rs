use std::collections::HashMap;

use tracing::debug;

use crate::device::pio::PioDevice;
use crate::pci::bus::PciBus;
use crate::pci::device::PciDevice;
use crate::pci::host_bridge::config_address::ConfigAddress;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_DATA: u16 = 0xcfc;

mod config_address {
    #[derive(Default)]
    pub struct ConfigAddress(u32);

    impl ConfigAddress {
        pub fn write(&mut self, buf: &[u8]) {
            let mut val = self.0.to_le_bytes();
            val[0..0 + buf.len()].copy_from_slice(buf);
            self.0 = u32::from_le_bytes(val);
        }

        pub fn read(&mut self, buf: &mut [u8]) {
            let bytes = self.0.to_le_bytes();
            buf.copy_from_slice(&bytes[0..buf.len()]);
        }

        pub fn enable(&self) -> bool {
            (self.0 & 0x8000_0000) != 0
        }

        pub fn bus(&self) -> u8 {
            ((self.0 >> 16) & 0xff) as u8
        }

        pub fn device(&self) -> u8 {
            ((self.0 >> 11) & 0x1f) as u8
        }

        pub fn function(&self) -> u8 {
            ((self.0 >> 8) & 0x07) as u8
        }

        pub fn register(&self) -> u8 {
            ((self.0 >> 2) & 0x3f) as u8
        }

        pub fn offset(&self) -> u8 {
            (self.0 & 0x3) as u8
        }
    }
}

#[derive(Default)]
pub struct PciHostBridge {
    bus: HashMap<u8, PciBus>,
    config_address: ConfigAddress,
}

impl PciHostBridge {
    fn get_device(&mut self, bus_number: u8, device_number: u8) -> Option<&mut Box<dyn PciDevice>> {
        debug!(bus_number, device_number);
        self.bus
            .get_mut(&bus_number)
            .and_then(|bus| bus.get_device_mut(device_number))
    }

    fn handle_out_config_data(&mut self, offset: u8, data: &[u8]) {
        assert_eq!(data.len(), 4);

        if !self.config_address.enable() {
            return;
        }

        let register = self.config_address.register();
        // let offset = self.config_address.offset();

        let start = register * 4 + offset;

        let Some(device) = self.get_device(self.config_address.bus(), self.config_address.device())
        else {
            return;
        };

        let configuration_space = device.get_configuration_space_mut();
        configuration_space.write(start, data);
    }

    fn handle_in_config_data(&mut self, offset: u8, data: &mut [u8]) {
        if !self.config_address.enable() {
            data.fill(0xff);

            return;
        }

        let register = self.config_address.register();
        // let offset = self.config_address.offset(); // ignore offset?
        let start = register * 4 + offset;

        let Some(device) = self.get_device(self.config_address.bus(), self.config_address.device())
        else {
            // When a configuration access attempts to select a device that does not exist,
            // the host bridge will complete the access without error, dropping all data on
            // writes and returning all ones on reads.
            data.fill(0xff);
            return;
        };

        let configuration_space = device.get_configuration_space();
        configuration_space.read(start, data);
    }
}

impl PioDevice for PciHostBridge {
    fn ports(&self) -> &[u16] {
        &[
            CONFIG_ADDRESS,
            CONFIG_ADDRESS + 1,
            CONFIG_ADDRESS + 2,
            CONFIG_ADDRESS + 3,
            CONFIG_DATA,
            CONFIG_DATA + 1,
            CONFIG_DATA + 2,
            CONFIG_DATA + 3,
        ]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        if port == CONFIG_ADDRESS {
            self.config_address.read(data);
        } else if port >= CONFIG_DATA && port < CONFIG_DATA + 4 {
            let offset = port - CONFIG_DATA;
            self.handle_in_config_data(offset as u8, data)
        } else {
            panic!("pci: 0x{:x}", port);
        }

        println!("in {:x}: {:?}", port, data);
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        println!("out {:x}: {:?}", port, data);

        if port == CONFIG_ADDRESS {
            self.config_address.write(data);
        } else if port >= CONFIG_DATA && port < CONFIG_DATA + 4 {
            let offset = port - CONFIG_DATA;
            self.handle_out_config_data(offset as u8, data);
        } else {
            panic!("pci: 0x{:x}", port);
        }
    }
}
