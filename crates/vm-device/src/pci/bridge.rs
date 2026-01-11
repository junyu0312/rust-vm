use crate::pci::configuration_space::ConfigurationSpace;
use crate::pci::device::PciDevice;

pub struct PciBridge {
    configuration_space: ConfigurationSpace,
}

impl PciDevice for PciBridge {
    fn get_configuration_space(&self) -> &ConfigurationSpace {
        &self.configuration_space
    }

    fn get_configuration_space_mut(&mut self) -> &mut ConfigurationSpace {
        &mut self.configuration_space
    }
}
