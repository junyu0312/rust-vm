use std::path::Path;

use crate::container::VfioContainer;
use crate::error::Result;

pub struct VfioDevice {
    device: vfio_ioctls::VfioDevice,
}

impl VfioDevice {
    pub fn new(path: &Path, container: &VfioContainer) -> Result<Self> {
        let device = vfio_ioctls::VfioDevice::new(path, container.container.clone())?;

        Ok(VfioDevice { device })
    }

    pub fn reset(&self) {
        self.device.reset();
    }
}
