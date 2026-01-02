use std::collections::HashMap;

use crate::pci::device::PciDevice;

pub struct PciBus {
    devices: HashMap<u8, Box<dyn PciDevice>>,
}
