use vm_core::device::mmio::MmioRange;

use crate::types::configuration_space::ConfigurationSpace;

pub mod type0;
pub mod type1;

pub trait BarHandler {
    fn read(&self, offset: u64, len: usize, data: &mut [u8]);

    fn write(&self, offset: u64, len: usize, data: &[u8]);
}

pub trait PciTypeFunctionCommon {
    const VENDOR_ID: u16;
    const DEVICE_ID: u16;
    const PROG_IF: u8;
    const SUBCLASS: u8;
    const CLASS_CODE: u8;
    const IRQ_LINE: u8;
    const IRQ_PIN: u8;

    fn init_capability(_configuration_space: &mut ConfigurationSpace) {
        // Default impl
    }
}

pub enum Callback {
    Void,
    // bar n, pci address range, handler
    RegisterBarClosure((u8, MmioRange, Box<dyn BarHandler>)),
}

pub trait PciFunction {
    fn write_bar(&self, n: u8, buf: &[u8]) -> Callback;

    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Callback;

    fn bar_handler(&self, bar: u8) -> Box<dyn BarHandler>;
}
