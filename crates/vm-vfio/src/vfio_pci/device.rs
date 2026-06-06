use std::iter;

use vm_core::device::Device;
use vm_pci::device::function::type0::Type0Function;
use vm_pci::types::device::PciDevice;
use vm_pci::types::function::PciFunction;

use crate::error::Result;
use crate::vfio::device::VfioDevice;
use crate::vfio_pci::function::VfioPciFunction;

pub struct VfioPciDevice {
    name: String,
    _vfio_device: VfioDevice,
    function: Type0Function<VfioPciFunction>,
}

impl VfioPciDevice {
    pub fn new(name: String, vfio_device: VfioDevice) -> Result<Self> {
        vfio_device.reset()?;

        let function = Type0Function::new(VfioPciFunction)?;

        Ok(VfioPciDevice {
            name,
            _vfio_device: vfio_device,
            function,
        })
    }
}

impl Device for VfioPciDevice {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl PciDevice for VfioPciDevice {
    fn get_function(&self, function: u8) -> Option<&dyn PciFunction> {
        if function == 0 {
            return Some(&self.function);
        }

        None
    }

    fn get_function_mut(&mut self, function: u8) -> Option<&mut dyn PciFunction> {
        if function == 0 {
            return Some(&mut self.function);
        }

        None
    }

    fn functions(&self) -> Box<dyn Iterator<Item = &(dyn PciFunction + '_)> + '_> {
        Box::new(iter::once(&self.function as &dyn PciFunction))
    }
}
