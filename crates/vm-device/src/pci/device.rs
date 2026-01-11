use crate::pci::configuration_space::ConfigurationSpace;

pub trait PciDevice {
    fn get_configuration_space(&self) -> &ConfigurationSpace;

    fn get_configuration_space_mut(&mut self) -> &mut ConfigurationSpace;
}
