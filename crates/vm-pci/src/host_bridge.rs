use crate::device::PciDevice;
use crate::types::configuration_space::capability::Capability;
use crate::types::function::BarHandler;
use crate::types::function::PciTypeFunctionCommon;
use crate::types::function::type0::Bar;
use crate::types::function::type0::PciType0Function;
use crate::types::function::type0::Type0Function;

struct HostBridgeFunction;

impl PciTypeFunctionCommon for HostBridgeFunction {
    const VENDOR_ID: u16 = 0x1b36; // From qemu log
    const DEVICE_ID: u16 = 0x0008; // From qemu log
    const CLASS_CODE: u32 = 0x060000;
    const IRQ_LINE: u8 = 0xff;
    const IRQ_PIN: u8 = 0x00;

    fn capabilities(&self) -> Vec<Capability> {
        vec![]
    }
}

impl PciType0Function for HostBridgeFunction {
    const BAR_SIZE: [Option<u32>; 6] = [None, None, None, None, None, None];

    fn bar_handler(&self, _bar: Bar) -> Option<Box<dyn BarHandler>> {
        None
    }
}

pub fn new_host_bridge() -> PciDevice {
    let function = Type0Function::new(HostBridgeFunction);
    PciDevice::from_single_function(Box::new(function))
}
