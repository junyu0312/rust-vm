use vm_core::device::mmio::MmioRange;

use crate::device::function::BarHandler;

mod type0;

pub enum EcamUpdateCallback {
    UpdateMmioRouter {
        bar: u8,
        pci_address_range: MmioRange,
        handler: Box<dyn BarHandler>,
    },
}

pub trait PciFunction {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback>;
}
