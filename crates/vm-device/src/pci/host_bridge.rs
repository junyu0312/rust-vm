use crate::pci::configuration_space::ConfigurationSpace;
use crate::pci::device::PciDevice;
use crate::pci::device::PciFunc;

const PCI_CLASS_BRIDGE_HOST: u16 = 0x0600;

pub struct PciHostBridgeFunc {
    cfg: ConfigurationSpace,
}

impl PciFunc for PciHostBridgeFunc {
    fn get_configuration_space(&self) -> &ConfigurationSpace {
        &self.cfg
    }

    fn get_configuration_space_mut(&mut self) -> &mut ConfigurationSpace {
        &mut self.cfg
    }
}

pub struct PciHostBridge {
    func: Vec<PciHostBridgeFunc>,
}

impl Default for PciHostBridge {
    fn default() -> Self {
        let cfg = ConfigurationSpace::new(PCI_CLASS_BRIDGE_HOST);

        Self {
            func: vec![PciHostBridgeFunc { cfg }],
        }
    }
}

impl PciDevice for PciHostBridge {
    fn get_func(&self, func: u8) -> &dyn PciFunc {
        &self.func[func as usize]
    }

    fn get_func_mut(&mut self, func: u8) -> &mut dyn PciFunc {
        &mut self.func[func as usize]
    }
}
