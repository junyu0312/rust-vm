use crate::pci::configuration_space::ConfigurationSpace;
use crate::pci::device::PciDevice;

const PCI_CLASS_BRIDGE_HOST: u16 = 0x0600;

pub struct PciHostBridge {
    cfg: ConfigurationSpace,
}

impl Default for PciHostBridge {
    fn default() -> Self {
        let cfg = ConfigurationSpace::new(PCI_CLASS_BRIDGE_HOST);

        Self { cfg }
    }
}

impl PciDevice for PciHostBridge {
    fn get_configuration_space(&self) -> &ConfigurationSpace {
        &self.cfg
    }

    fn get_configuration_space_mut(&mut self) -> &mut ConfigurationSpace {
        &mut self.cfg
    }
}
