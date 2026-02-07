use tracing::debug;

use crate::pci::bus::PciBus;
use crate::pci::device::PciDevice;
use crate::pci::host_bridge::new_host_bridge;

pub mod mmio;
pub mod pio;

struct PciRootComplex {
    host_bridge: PciDevice,
    bus: Vec<PciBus>,
    allocation: usize,
}

impl Default for PciRootComplex {
    fn default() -> Self {
        Self {
            host_bridge: new_host_bridge(),
            bus: vec![PciBus::default()],
            allocation: 1, // 0 is reserved for host bridge
        }
    }
}

impl PciRootComplex {
    pub fn register_device(&mut self, device: PciDevice) -> Result<(), PciDevice> {
        let bus_number = self.allocation / 32;
        let device_number = self.allocation % 32;
        self.allocation += 1;

        assert_eq!(bus_number, 0);

        self.bus[0].register_device(device_number as u8, device);

        Ok(())
    }

    fn get_device(&self, bus_number: u8, device_number: u8) -> Option<&PciDevice> {
        if bus_number == 0 && device_number == 0 {
            return Some(&self.host_bridge);
        }

        self.bus
            .get(bus_number as usize)
            .and_then(|bus| bus.get_device(device_number))
    }

    fn get_device_mut(&mut self, bus_number: u8, device_number: u8) -> Option<&mut PciDevice> {
        if bus_number == 0 && device_number == 0 {
            return Some(&mut self.host_bridge);
        }

        self.bus
            .get_mut(bus_number as usize)
            .and_then(|bus| bus.get_device_mut(device_number))
    }

    fn handle_ecam_read(&self, bus: u8, device: u8, func: u8, offset: u16, data: &mut [u8]) {
        let Some(function) = self
            .get_device(bus, device)
            .and_then(|dev| dev.get_func(func))
        else {
            // When a configuration access attempts to select a device that does not exist,
            // the host bridge will complete the access without error, dropping all data on
            // writes and returning all ones on reads.
            data.fill(0xff);
            return;
        };

        function.ecam_read(offset, data);

        debug!(bus, device, func, offset, ?data, "ecam read");
    }

    fn handle_ecam_write(&mut self, bus: u8, device: u8, func: u8, offset: u16, data: &[u8]) {
        debug!(bus, device, func, offset, data, "ecam write");

        let Some(function) = self
            .get_device_mut(bus, device)
            .and_then(|dev| dev.get_func_mut(func))
        else {
            return;
        };

        function.ecam_write(offset, data);
    }
}
