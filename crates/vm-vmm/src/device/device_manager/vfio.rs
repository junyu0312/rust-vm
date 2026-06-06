use std::path::Path;

use vm_vfio::vfio::container::VfioContainer;
use vm_vfio::vfio::device::VfioDevice;
use vm_vfio::vfio_pci::device::VfioPciDevice;

use crate::device::device_manager::DeviceManager;
use crate::device::error::InitDeviceError;

impl DeviceManager {
    pub fn init_vfio(&mut self) -> Result<(), InitDeviceError> {
        let vfio_container = VfioContainer::new()?;

        self.vfio_container = Some(vfio_container);

        Ok(())
    }

    pub fn init_vfio_device(
        &self,
        name: String,
        container: &VfioContainer,
        path: &Path,
    ) -> Result<VfioPciDevice, InitDeviceError> {
        let vfio_device = VfioDevice::new(path, container)?;

        let vfio_pci_device = VfioPciDevice::new(name, vfio_device)?;

        Ok(vfio_pci_device)
    }
}
