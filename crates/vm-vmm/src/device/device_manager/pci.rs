use vm_pci::root_complex::pci_root_complex::PciRootComplex;

use crate::device::device_manager::DeviceManager;
use crate::device::error::InitDeviceError;

impl DeviceManager {
    pub fn init_pci_root_complex(&mut self) -> Result<PciRootComplex, InitDeviceError> {
        Ok(PciRootComplex::default())
    }
}
