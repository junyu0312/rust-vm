use crate::error::Error;
use crate::error::Result;
use crate::vfio::VfioContainerOps;

pub struct VfioContainerDummy;

impl VfioContainerDummy {
    pub fn new() -> Result<Self> {
        Ok(VfioContainerDummy)
    }
}

impl VfioContainerOps for VfioContainerDummy {
    fn new_device(&self, _path: &std::path::Path) -> Result<Box<dyn crate::vfio::VfioDeviceOps>> {
        Err(Error::NotSupport)
    }

    unsafe fn vfio_dma_map(&self, _iova: u64, _size: usize, _user_addr: *mut u8) -> Result<()> {
        Err(Error::NotSupport)
    }

    fn vfio_dma_unmap(&self, _iova: u64, _size: usize) -> Result<()> {
        Err(Error::NotSupport)
    }
}
