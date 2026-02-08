use crate::pci::device::PciDevice;
use crate::pci::types::function::PciTypeFunctionCommon;
use crate::pci::types::function::type0::PciType0Function;
use crate::pci::types::function::type0::Type0Function;
struct HostBridgeFunction;

impl PciTypeFunctionCommon for HostBridgeFunction {
    const VENDOR_ID: u16 = 0x1b36; // From qemu log
    const DEVICE_ID: u16 = 0x0008; // From qemu log
    const PROG_IF: u8 = 0x00;
    const SUBCLASS: u8 = 0x00;
    const CLASS_CODE: u8 = 0x06;
}
impl PciType0Function for HostBridgeFunction {
    const BAR_SIZE: [Option<u32>; 6] = [None, None, None, None, None, None];
}

pub fn new_host_bridge() -> PciDevice {
    let function = Type0Function::<HostBridgeFunction>::default();
    PciDevice::new(vec![Box::new(function)])
}
