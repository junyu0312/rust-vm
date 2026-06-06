use std::sync::Arc;

use crate::error::Result;

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

impl VfioContainer {
    /// # Safety
    pub unsafe fn vfio_dma_map(&self, iova: u64, size: usize, user_addr: *mut u8) -> Result<()> {
        unsafe {
            self.container.vfio_dma_map(iova, size, user_addr)?;
        }

        Ok(())
    }

    pub fn vfio_dma_unmap(&self, iova: u64, size: usize) -> Result<()> {
        self.container.vfio_dma_unmap(iova, size)?;

        Ok(())
    }
}
