use crate::error::Error;
use crate::types::configuration_space::ConfigurationSpace;

pub mod type0;

pub trait BarHandler {
    fn read(&self, offset: u64, data: &mut [u8]);

    fn write(&self, offset: u64, data: &[u8]);
}

pub trait PciTypeFunctionCommon {
    const VENDOR_ID: u16;
    const DEVICE_ID: u16;
    const CLASS_CODE: u32;

    /// legacy irq_line, irq_pin
    fn legacy_interrupt(&self) -> Option<(u8, u8)>;

    fn init_capability(&self, cfg: &mut ConfigurationSpace) -> Result<(), Error>;
}
