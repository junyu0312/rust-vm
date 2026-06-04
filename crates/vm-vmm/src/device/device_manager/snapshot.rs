use std::collections::BTreeMap;
use std::io::Cursor;

use serde::Deserialize;
use serde::Serialize;
use tracing::warn;
use vm_core::device::error::DeviceSnapshotError;

use crate::device::device_manager::DeviceManager;

#[derive(Serialize, Deserialize)]
pub struct DeviceSnapshot {
    // TODO: use name is not elegant
    devices: BTreeMap<String, Vec<u8>>,
}

impl DeviceManager {
    pub fn build_snapshot(&self) -> Result<DeviceSnapshot, DeviceSnapshotError> {
        let mut devices = BTreeMap::default();

        {
            for device in &self.pio_manager.device {
                let mut buf = vec![];
                device.save(&mut buf)?;
                let old = devices.insert(device.name(), buf);

                assert!(old.is_none());
            }
        }

        {
            for device in self.mmio_devices() {
                let mut buf = vec![];
                device.save(&mut buf)?;
                let old = devices.insert(device.name(), buf);

                assert!(old.is_none());
            }
        }

        let snap = DeviceSnapshot { devices };

        Ok(snap)
    }

    pub fn install_snapshot(&mut self, snap: DeviceSnapshot) -> Result<(), DeviceSnapshotError> {
        for device in &mut self.pio_manager.device {
            let Some(buf) = snap.devices.get(&device.name()) else {
                warn!(name = device.name(), "device snapshot not found, skipped");
                continue;
            };

            device.load(&mut Cursor::new(buf))?;
        }

        for device in self.mmio_devices_mut() {
            let Some(buf) = snap.devices.get(&device.name()) else {
                warn!(name = device.name(), "device snapshot not found, skipped");
                continue;
            };

            device.load(&mut Cursor::new(buf))?;
        }

        Ok(())
    }
}
