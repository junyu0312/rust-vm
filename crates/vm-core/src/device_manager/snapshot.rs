use std::io::Read;
use std::io::Write;

use vm_snapshot::ops::Snapshotable;

use crate::device::error::DeviceSnapshotError;
use crate::device_manager::DeviceManager;

impl Snapshotable for DeviceManager {
    type Error = DeviceSnapshotError;

    fn save(&self, writer: &mut dyn Write) -> Result<(), Self::Error> {
        {
            let it = self.pio_manager.address_space.iter();
            writer.write_all(&(it.len() as u64).to_le_bytes())?;

            for (port, (len, idx)) in it {
                writer.write_all(&(port).to_le_bytes())?;
                writer.write_all(&(*len as u64).to_le_bytes())?;

                let device = self.pio_manager.device.get(*idx).unwrap();
                let Some(device_snapshot) = device.support_snapshot() else {
                    return Err(DeviceSnapshotError::DeviceNotSupportSnapshot(device.name()));
                };

                device_snapshot.save(writer)?;
            }
        }

        {
            let it = self.mmio_manager.devices.iter();
            writer.write_all(&(it.len() as u64).to_le_bytes())?;

            for device in it {
                let Some(device_snapshot) = device.support_snapshot() else {
                    return Err(DeviceSnapshotError::DeviceNotSupportSnapshot(device.name()));
                };

                device_snapshot.save(writer)?;
            }
        }

        Ok(())
    }

    fn restore(&mut self, _reader: &mut dyn Read) -> Result<(), Self::Error> {
        todo!()
    }
}
