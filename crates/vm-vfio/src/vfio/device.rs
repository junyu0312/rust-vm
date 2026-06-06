use std::path::Path;

use crate::error::Result;
use crate::vfio::container::VfioContainer;

pub struct VfioDevice {
    device: vfio_ioctls::VfioDevice,
}

impl VfioDevice {
    pub fn new(path: &Path, container: &VfioContainer) -> Result<VfioDevice> {
        let device = VfioDevice {
            device: vfio_ioctls::VfioDevice::new(path, container.container.clone())?,
        };

        Ok(device)
    }

    pub(crate) fn reset(&self) -> Result<()> {
        self.device.reset();

        Ok(())
    }
}
