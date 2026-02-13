use vm_core::device::mmio::MmioRange;

use crate::device::function::BarHandler;

mod type0;

pub enum EcamUpdateCallback {
    // bar n, pci address range, handler
    UpdateMmioRouter((u8, MmioRange, Box<dyn BarHandler>)),
}

pub trait PciFunction {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback>;
}
