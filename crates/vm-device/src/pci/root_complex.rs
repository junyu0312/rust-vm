use std::collections::HashMap;

use tracing::debug;
use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

use crate::pci::bus::PciBus;
use crate::pci::config_address::ConfigAddress;
use crate::pci::device::PciDevice;
use crate::pci::host_bridge::PciHostBridge;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_DATA: u16 = 0xcfc;

#[derive(Default)]
pub struct PciRootComplex {
    bus: HashMap<u8, PciBus>,
    host_bridge: Box<PciHostBridge>,
    config_address: ConfigAddress,
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

    fn handle_out_config_address(&mut self, offset: u8, data: &[u8]) {
        self.config_address.write(offset, data);
    }

    fn handle_in_config_address(&mut self, offset: u8, data: &mut [u8]) {
        self.config_address.read(offset, data);
    }

    fn handle_out_config_data(&mut self, offset: u8, data: &[u8]) {
        assert_eq!(data.len(), 4);

        if !self.config_address.enable() {
            return;
        }

        let register = self.config_address.register();
        // let offset = self.config_address.offset();

        let start = register * 4 + offset;

        let Some(device) =
            self.get_device_mut(self.config_address.bus(), self.config_address.device())
        else {
            return;
        };

        let configuration_space = device.get_configuration_space_mut();
        configuration_space.write(start, data);
    }

    fn handle_in_config_data(&self, offset: u8, data: &mut [u8]) {
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

impl Device for PciRootComplex {
    fn name(&self) -> String {
        "pci_root_complex".to_string()
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for PciRootComplex {
    fn ports(&self) -> Vec<PortRange> {
        vec![
            PortRange {
                start: CONFIG_ADDRESS,
                len: 4,
            },
            PortRange {
                start: CONFIG_DATA,
                len: 4,
            },
        ]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        if (CONFIG_ADDRESS..CONFIG_ADDRESS + 4).contains(&port) {
            let offset = port - CONFIG_ADDRESS;
            self.handle_in_config_address(offset as u8, data);
        } else if (CONFIG_DATA..CONFIG_DATA + 4).contains(&port) {
            let offset = port - CONFIG_DATA;
            self.handle_in_config_data(offset as u8, data)
        } else {
            panic!("pci: 0x{:x}", port);
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        if (CONFIG_ADDRESS..CONFIG_ADDRESS + 4).contains(&port) {
            let offset = port - CONFIG_ADDRESS;
            self.handle_out_config_address(offset as u8, data);
        } else if (CONFIG_DATA..CONFIG_DATA + 4).contains(&port) {
            let offset = port - CONFIG_DATA;
            self.handle_out_config_data(offset as u8, data);
        } else {
            panic!("pci: 0x{:x}", port);
        }
    }
}
