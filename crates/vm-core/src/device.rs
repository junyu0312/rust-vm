use std::io::Read;
use std::io::Write;

use acpi_tables::Aml;

use crate::device::error::DeviceSnapshotError;
use crate::device::mmio::mmio_device::MmioDevice;
use crate::device::pio::pio_device::PioDevice;

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

    fn support_aml(&self) -> Option<&dyn Aml> {
        None
    }

    fn support_pio_transport(&self) -> Option<&dyn PioDevice> {
        None
    }

    fn support_pio_transport_mut(&mut self) -> Option<&mut dyn PioDevice> {
        None
    }

    fn support_mmio_transport(&self) -> Option<&dyn MmioDevice> {
        None
    }

    fn support_mmio_transport_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        None
    }
}
