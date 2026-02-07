use crate::pci::types::function::PciFunction;

pub struct PciDevice {
    functions: Vec<Box<dyn PciFunction>>,
}

impl PciDevice {
    pub fn new(functions: Vec<Box<dyn PciFunction>>) -> Self {
        PciDevice { functions }
    }

    pub fn get_func(&self, func: u8) -> Option<&dyn PciFunction> {
        self.functions.get(func as usize).map(|r| r.as_ref() as _)
    }

    pub fn get_func_mut(&mut self, func: u8) -> Option<&mut dyn PciFunction> {
        self.functions
            .get_mut(func as usize)
            .map(|r| r.as_mut() as _)
    }
}
