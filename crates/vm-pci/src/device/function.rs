use crate::device::capability::Capability;

pub mod type0;

pub trait BarHandler {
    fn read(&self, offset: u64, data: &mut [u8]);

    fn write(&self, offset: u64, data: &[u8]);
}

pub trait PciTypeFunctionCommon {
    const VENDOR_ID: u16;
    const DEVICE_ID: u16;
    const CLASS_CODE: u32;
    const IRQ_LINE: u8;
    const IRQ_PIN: u8;

    // fn init_capability(_configuration_space: &mut ConfigurationSpace);
    fn capabilities(&self) -> Vec<Capability>;
}
