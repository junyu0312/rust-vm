use std::path::Path;

use crate::error::Result;

#[cfg(not(target_os = "linux"))]
mod dummy;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
pub use dummy::container::VfioContainerDummy as VfioContainer;
#[cfg(target_os = "linux")]
pub use linux::container::VfioContainer;

pub trait VfioContainerOps {
    fn new_device(&self, path: &Path) -> Result<Box<dyn VfioDeviceOps>>;

    /// # Safety
    unsafe fn vfio_dma_map(&self, iova: u64, size: usize, user_addr: *mut u8) -> Result<()>;

    fn vfio_dma_unmap(&self, iova: u64, size: usize) -> Result<()>;
}

pub trait VfioDeviceOps: Send + Sync {
    fn reset(&self) -> Result<()>;
}
