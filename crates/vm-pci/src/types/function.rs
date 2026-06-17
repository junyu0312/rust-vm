use std::ops::Range;

use crate::types::interrupt::InterruptMapEntry;

pub mod type0;

pub enum EcamUpdateCallbackOps {
    AddPioRouter {
        bar: u8,
        port: Range<u16>,
    },
    RemovePioRouter {
        bar: u8,
    },
    AddMmioRouter {
        bar: u8,
        pci_address_range: Range<u64>,
    },
    RemoveMmioRouter {
        bar: u8,
    },
}

pub struct EcamUpdateCallback(pub Vec<EcamUpdateCallbackOps>);

pub trait PciFunction: PciFunctionArch + Send + Sync {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback>;

    fn bar_read(&self, bar: u8, offset: u64, buf: &mut [u8]);

    fn bar_write(&self, bar: u8, offset: u64, buf: &[u8]);

    // irq_line, irq_pin
    fn legacy_irq(&self) -> Option<(u8, u8)>;
}

pub trait PciFunctionArch {
    fn interrupt_map_entry(&self, bus: u8, device: u8, function: u8) -> Option<InterruptMapEntry>;
}
