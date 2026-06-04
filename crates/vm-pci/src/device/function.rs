use crate::error::Error;
use crate::types::configuration_space::ConfigurationSpace;

pub mod type0;

pub trait BarHandler: Send + Sync {
    fn read(&self, offset: u64, data: &mut [u8]);

    fn write(&self, offset: u64, data: &[u8]);
}

pub trait PciTypeFunctionCommon: Send {
    fn vendor_id(&self) -> u16;

    fn device_id(&self) -> u16;

    fn class_code(&self) -> u32;

    /// legacy irq_line, irq_pin
    fn legacy_interrupt(&self) -> Option<(u8, u8)>;

    fn init_capability(&self, cfg: &mut ConfigurationSpace) -> Result<(), Error>;
}
