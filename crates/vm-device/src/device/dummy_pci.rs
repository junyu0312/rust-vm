use vm_virtio::transport::pci::pci_header::VENDOR_ID;
use vm_virtio::transport::pci::pci_header::VirtIoPciDeviceId;

use crate::pci::types::function::PciTypeFunctionCommon;
use crate::pci::types::function::type0::PciType0Function;

pub struct DummyPci;

impl PciTypeFunctionCommon for DummyPci {
    const VENDOR_ID: u16 = VENDOR_ID;
    const DEVICE_ID: u16 = VirtIoPciDeviceId::Blk as u16;
    const PROG_IF: u8 = 0;
    const SUBCLASS: u8 = 0x80;
    const CLASS_CODE: u8 = 0x01;
}

impl PciType0Function for DummyPci {
    const BAR_SIZE: [Option<u32>; 6] = [Some(0x1000), None, None, None, None, None];
}
