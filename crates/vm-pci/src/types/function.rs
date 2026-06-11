use std::ops::Range;

use crate::device::function::BarHandler;
use crate::types::interrupt::InterruptMapEntry;

pub mod type0;

pub enum EcamUpdateCallback {
    UpdateMmioRouter {
        bar: u8,
        pci_address_range: Range<u64>,
        handler: Box<dyn BarHandler>,
    },
}

pub trait PciFunction: PciFunctionArch + Send + Sync {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback>;
}

pub trait PciFunctionArch {
    fn interrupt_map_entry(&self, bus: u8, device: u8, function: u8) -> Option<InterruptMapEntry>;
}
