use vm_core::device::mmio::MmioRange;

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

pub trait PciFunction: PciFunctionArch {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback>;
}

pub trait PciFunctionArch {
    fn interrupt_map_entry(&self, bus: u8, device: u8, function: u8) -> Option<InterruptMapEntry>;
}
