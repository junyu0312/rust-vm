use crate::pci::configuration_space::ConfigurationSpace;

pub trait PciFunc {
    fn get_configuration_space(&self) -> &ConfigurationSpace;

    fn get_configuration_space_mut(&mut self) -> &mut ConfigurationSpace;
}

pub trait PciDevice {
    fn get_func(&self, func: u8) -> &dyn PciFunc;

    fn get_func_mut(&mut self, func: u8) -> &mut dyn PciFunc;
}
