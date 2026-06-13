use std::path::Path;

use vfio_ioctls::VfioRegionInfoCap;

use crate::error::Error;
use crate::error::Result;
use crate::vfio::container::VfioContainer;

pub struct VfioRegionInfo {
    pub(crate) flags: u32,
    pub(crate) _caps: Vec<VfioRegionInfoCap>,
    pub(crate) size: u64,
    pub(crate) _offset: u64,
}

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

    pub(crate) fn num_regions(&self) -> usize {
        self.device.num_regions() as usize
    }

    pub(crate) fn get_region_info(&self, index: u32) -> Result<VfioRegionInfo> {
        if index as usize >= self.num_regions() {
            return Err(Error::RegionNotExists(index as usize));
        }

        let flags = self.device.get_region_flags(index);
        let caps = self.device.get_region_caps(index);
        let size = self.device.get_region_size(index);
        let offset = self.device.get_region_offset(index);

        Ok(VfioRegionInfo {
            flags,
            _caps: caps,
            size,
            _offset: offset,
        })
    }

    pub(crate) fn region_read(&self, index: u32, buf: &mut [u8], addr: u64) -> Result<()> {
        self.device.region_read(index, buf, addr);

        Ok(())
    }

    pub(crate) fn region_write(&self, index: u32, buf: &[u8], addr: u64) -> Result<()> {
        self.device.region_write(index, buf, addr);

        Ok(())
    }
}
