use std::path::Path;
use std::sync::Arc;

use vfio_ioctls::VfioDevice;

use crate::error::Result;
use crate::vfio::VfioContainerOps;
use crate::vfio::VfioDeviceOps;

pub struct VfioContainer {
    pub(crate) container: Arc<vfio_ioctls::VfioContainer>,
}

impl VfioContainer {
    pub fn new() -> Result<Self> {
        let container = vfio_ioctls::VfioContainer::new(None)?;

        Ok(VfioContainer {
            container: container.into(),
        })
    }
}

impl VfioContainerOps for VfioContainer {
    fn new_device(&self, path: &Path) -> Result<Box<dyn VfioDeviceOps>> {
        let device = crate::vfio::linux::device::VfioDevice {
            device: VfioDevice::new(path, self.container.clone())?,
        };

        Ok(Box::new(device))
    }

    unsafe fn vfio_dma_map(&self, iova: u64, size: usize, user_addr: *mut u8) -> Result<()> {
        unsafe {
            self.container.vfio_dma_map(iova, size, user_addr)?;
        }

        Ok(())
    }

    fn vfio_dma_unmap(&self, iova: u64, size: usize) -> Result<()> {
        self.container.vfio_dma_unmap(iova, size)?;

        Ok(())
    }
}
