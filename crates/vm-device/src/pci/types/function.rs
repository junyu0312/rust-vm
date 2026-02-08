use zerocopy::FromBytes;

use crate::pci::types::configuration_space::ConfigurationSpace;
use crate::pci::types::configuration_space::common::HeaderCommon;

pub mod type0;
pub mod type1;

pub trait PciTypeFunctionCommon {
    const VENDOR_ID: u16;
    const DEVICE_ID: u16;
    const SUBCLASS: u8;
    const CLASS_CODE: u8;
    const PROG_IF: u8;

    fn new_configuration_space(header_type: u8) -> ConfigurationSpace {
        let mut buf = [0; 4096];
        let header = HeaderCommon::mut_from_bytes(&mut buf[0..size_of::<HeaderCommon>()]).unwrap();

        header.vendor_id = Self::VENDOR_ID;
        header.device_id = Self::DEVICE_ID;
        header.prog_if = Self::PROG_IF;
        header.subclass = Self::SUBCLASS;
        header.class_code = Self::CLASS_CODE;
        header.header_type = header_type;

        ConfigurationSpace { buf }
    }
}

pub trait PciFunction {
    fn write_bar(&mut self, n: usize, buf: &[u8]);

    fn ecam_read(&self, offset: u16, buf: &mut [u8]);

    fn ecam_write(&mut self, offset: u16, buf: &[u8]);
}
