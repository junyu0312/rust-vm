use crate::error::Result;
use crate::vfio::VfioDeviceOps;

pub struct VfioDevice {
    pub(crate) device: vfio_ioctls::VfioDevice,
}

impl VfioDeviceOps for VfioDevice {
    fn reset(&self) -> Result<()> {
        self.device.reset();

        Ok(())
    }
}
