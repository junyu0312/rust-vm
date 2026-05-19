use std::io::Read;
use std::io::Write;

use crate::device::error::DeviceSnapshotError;

pub mod error;
pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(self.name()))
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(self.name()))
    }

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(self.name()))
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(self.name()))
    }
}
