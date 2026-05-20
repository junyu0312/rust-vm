use std::io::Read;
use std::io::Write;

use vm_core::device::error::DeviceSnapshotError;
use vm_core::device::mmio::layout::MmioRange;

use crate::device::function::BarHandler;
use crate::types::interrupt::InterruptMapEntry;

mod type0;

pub enum EcamUpdateCallback {
    UpdateMmioRouter {
        bar: u8,
        pci_address_range: MmioRange,
        handler: Box<dyn BarHandler>,
    },
}

pub trait PciFunction: PciFunctionArch + Send + Sync {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback>;

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            "unknown device".to_string(),
        ))
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            "unknown device".to_string(),
        ))
    }
}

pub trait PciFunctionArch {
    fn interrupt_map_entry(&self, bus: u8, device: u8, function: u8) -> Option<InterruptMapEntry>;
}
