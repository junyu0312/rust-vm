use std::collections::BTreeMap;
use std::io::Cursor;

use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

use crate::device::error::DeviceSnapshotError;
use crate::device_manager::DeviceManager;

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
                let Some(snapshot_cap) = device.support_snapshot() else {
                    return Err(DeviceSnapshotError::DeviceNotSupportSnapshot(device.name()));
                };

                let mut buf = vec![];
                snapshot_cap.save(&mut buf)?;
                let old = devices.insert(device.name(), buf);

                assert!(old.is_none());
            }
        }

        {
            for device in self.mmio_devices() {
                let Some(snapshot_cap) = device.support_snapshot() else {
                    return Err(DeviceSnapshotError::DeviceNotSupportSnapshot(device.name()));
                };

                let mut buf = vec![];
                snapshot_cap.save(&mut buf)?;
                let old = devices.insert(device.name(), buf);

                assert!(old.is_none());
            }
        }

        let snap = DeviceSnapshot { devices };

        Ok(snap)
    }

    pub fn install_snapshot(&mut self, snap: DeviceSnapshot) -> Result<(), DeviceSnapshotError> {
        for device in &mut self.pio_manager.device {
            let Some(snapshot_cap) = device.support_snapshot() else {
                return Err(DeviceSnapshotError::DeviceNotSupportSnapshot(device.name()));
            };

            let Some(buf) = snap.devices.get(&device.name()) else {
                warn!(name = device.name(), "device snapshot not found, skipped");
                continue;
            };

            snapshot_cap.restore(&mut Cursor::new(buf))?;
        }

        for device in self.mmio_devices() {
            let Some(snapshot_cap) = device.support_snapshot() else {
                return Err(DeviceSnapshotError::DeviceNotSupportSnapshot(device.name()));
            };

            let Some(buf) = snap.devices.get(&device.name()) else {
                warn!(name = device.name(), "device snapshot not found, skipped");
                continue;
            };

            snapshot_cap.restore(&mut Cursor::new(buf))?;
        }

        Ok(())
    }
}
