use crate::device::PciDevice;
use crate::types::function::BarHandler;
use crate::types::function::PciTypeFunctionCommon;
use crate::types::function::type0::PciType0Function;
use crate::types::function::type0::Type0Function;

struct HostBridgeFunction;

impl PciTypeFunctionCommon for HostBridgeFunction {
    const VENDOR_ID: u16 = 0x1b36; // From qemu log
    const DEVICE_ID: u16 = 0x0008; // From qemu log
    const PROG_IF: u8 = 0x00;
    const SUBCLASS: u8 = 0x00;
    const CLASS_CODE: u8 = 0x06;
    const IRQ_LINE: u8 = 0xff;
    const IRQ_PIN: u8 = 0x00;
}

impl PciType0Function for HostBridgeFunction {
    const BAR_SIZE: [Option<u32>; 6] = [None, None, None, None, None, None];

    fn bar_handler(&self, _n: u8) -> Box<dyn BarHandler> {
        todo!()
    }
}

pub fn new_host_bridge() -> PciDevice {
    let function = Type0Function::new(HostBridgeFunction);
    PciDevice::new(vec![Box::new(function)])
}
