use vm_core::device::mmio::MmioRange;

use crate::device::function::BarHandler;

mod type0;

pub enum Callback {
    Void,
    // bar n, pci address range, handler
    RegisterBarClosure((u8, MmioRange, Box<dyn BarHandler>)),
}

pub trait PciFunction {
    fn write_bar(&self, n: u8, buf: &[u8]) -> Callback;

    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Callback;
}
