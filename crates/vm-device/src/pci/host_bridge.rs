use crate::pci::device::PciDevice;
use crate::pci::types::function::PciTypeFunctionCommon;
use crate::pci::types::function::type1::PciType1Function;
use crate::pci::types::function::type1::Type1Function;

struct HostBridgeFunction;

impl PciTypeFunctionCommon for HostBridgeFunction {
    const VENDOR_ID: u16 = 0;
    const DEVICE_ID: u16 = 0;
    const SUBCLASS: u8 = 0x06;
}
impl PciType1Function for HostBridgeFunction {}

pub fn new_host_bridge() -> PciDevice {
    let function = Type1Function::<HostBridgeFunction>::default();
    PciDevice::new(vec![Box::new(function)])
}
