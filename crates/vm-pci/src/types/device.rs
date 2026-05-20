use vm_core::device::Device;

use crate::types::function::PciFunction;

pub trait PciDevice: Device {
    fn get_function(&self, function: u8) -> Option<&dyn PciFunction>;

    fn get_function_mut(&mut self, function: u8) -> Option<&mut dyn PciFunction>;

    fn functions(&self) -> Box<dyn Iterator<Item = &(dyn PciFunction + '_)> + '_>;
}
