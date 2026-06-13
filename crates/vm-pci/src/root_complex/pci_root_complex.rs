use std::io::Read;
use std::io::Write;
use std::sync::RwLock;

use tracing::debug;
use vm_core::device::error::DeviceSnapshotError;

use crate::bus::PciBus;
use crate::host_bridge::new_host_bridge;
use crate::root_complex::mmio_router::MmioRouter;

use crate::types::device::PciDevice;
use crate::types::function::EcamUpdateCallback;

pub struct PciRootComplex {
    pub(crate) bus: Vec<PciBus>,
    pub(crate) mmio_router: RwLock<MmioRouter>,
    allocation: usize,
}

impl Default for PciRootComplex {
    fn default() -> Self {
        let mut rc = PciRootComplex {
            bus: vec![PciBus::default()],
            mmio_router: Default::default(),
            allocation: 0,
        };

        rc.register_device(Box::new(new_host_bridge().unwrap()))
            .map_err(|_| "failed to register host bridge")
            .unwrap();

        rc
    }
}

impl PciRootComplex {
    pub fn register_device(
        &mut self,
        device: Box<dyn PciDevice>,
    ) -> Result<(), Box<dyn PciDevice>> {
        let bus_number = self.allocation / 32;
        let device_number = self.allocation % 32;
        self.allocation += 1;

        assert_eq!(bus_number, 0);

        self.bus[0].register_device(device_number as u8, device);

        Ok(())
    }

    pub fn get_device(&self, bus_number: u8, device_number: u8) -> Option<&dyn PciDevice> {
        self.bus
            .get(bus_number as usize)
            .and_then(|bus| bus.get_device(device_number))
    }

    pub fn handle_ecam_read(&self, bus: u8, device: u8, func: u8, offset: u16, data: &mut [u8]) {
        if let Some(dev) = self.get_device(bus, device)
            && let Some(function) = dev.get_function(func)
        {
            function.ecam_read(offset, data);

            debug!(bus, device, func, offset, ?data, "ecam read");
        } else {
            // When a configuration access attempts to select a device that does not exist,
            // the host bridge will complete the access without error, dropping all data on
            // writes and returning all ones on reads.
            data.fill(0xff);
        }
    }

    pub fn handle_ecam_write(&self, bus: u8, device: u8, func: u8, offset: u16, data: &[u8]) {
        debug!(bus, device, func, offset, data, "ecam write");

        let Some(function) = self
            .get_device(bus, device)
            .and_then(|dev| dev.get_function(func))
        else {
            return;
        };

        if let Some(cb) = function.ecam_write(offset, data) {
            match cb {
                EcamUpdateCallback::UpdateMmioRouter {
                    bar,
                    pci_address_range,
                } => self.mmio_router.write().unwrap().register_handler(
                    pci_address_range,
                    bus,
                    device,
                    func,
                    bar,
                ),
            }
        }
    }

    pub fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        for bus in &self.bus {
            for (_, device) in bus.devices() {
                device.save(writer)?;
            }
        }

        Ok(())
    }

    pub fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        for bus in &mut self.bus {
            for (_, device) in bus.devices_mut() {
                device.load(reader)?;
            }
        }

        Ok(())
    }
}
