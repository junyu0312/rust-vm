use crate::device::function::BarHandler;
use crate::device::function::PciTypeFunctionCommon;
use crate::device::function::type0::Bar;
use crate::device::function::type0::PciType0Function;
use crate::device::function::type0::Type0Function;
use crate::device::pci_device::PciDevice;
use crate::error::Error;
use crate::types::configuration_space::ConfigurationSpace;

struct HostBridgeFunction;

impl PciTypeFunctionCommon for HostBridgeFunction {
    const VENDOR_ID: u16 = 0x1b36; // From qemu log
    const DEVICE_ID: u16 = 0x0008; // From qemu log
    const CLASS_CODE: u32 = 0x060000;

    fn legacy_interrupt(&self) -> Option<(u8, u8)> {
        None
    }

    fn init_capability(&self, _cfg: &mut ConfigurationSpace) -> Result<(), Error> {
        Ok(())
    }
}

impl PciType0Function for HostBridgeFunction {
    const BAR_SIZE: [Option<u32>; 6] = [None, None, None, None, None, None];

    fn bar_handler(&self, _bar: Bar) -> Option<Box<dyn BarHandler>> {
        None
    }
}

pub fn new_host_bridge() -> Result<PciDevice, Error> {
    let function = Type0Function::new(HostBridgeFunction)?;
    Ok(PciDevice::from_single_function(Box::new(function)))
}
