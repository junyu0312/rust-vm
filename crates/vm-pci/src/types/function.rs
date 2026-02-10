use vm_core::device::mmio::MmioRange;
use zerocopy::FromBytes;

use crate::types::configuration_space::ConfigurationSpace;
use crate::types::configuration_space::common::HeaderCommon;

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

    fn new_configuration_space(header_type: u8) -> ConfigurationSpace {
        let mut buf = [0; 4096];
        let header = HeaderCommon::mut_from_bytes(&mut buf[0..size_of::<HeaderCommon>()]).unwrap();

        header.vendor_id = Self::VENDOR_ID;
        header.device_id = Self::DEVICE_ID;
        header.prog_if = Self::PROG_IF;
        header.subclass = Self::SUBCLASS;
        header.class_code = Self::CLASS_CODE;
        header.header_type = header_type;

        let mut cfg = ConfigurationSpace::new(buf);

        Self::init_capability(&mut cfg);

        cfg
    }

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
