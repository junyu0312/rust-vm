use std::path::Path;

use vfio_bindings::bindings::vfio::VFIO_PCI_INTX_IRQ_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_MSI_IRQ_INDEX;
use vfio_bindings::bindings::vfio::VFIO_PCI_MSIX_IRQ_INDEX;
use vfio_ioctls::VfioIrq;
use vfio_ioctls::VfioRegionInfoCap;
use vmm_sys_util::eventfd::EventFd;

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

    pub(crate) fn get_intx_irq_info(&self) -> Option<&VfioIrq> {
        self.device.get_irq_info(VFIO_PCI_INTX_IRQ_INDEX)
    }

    pub(crate) fn enable_intx(&self, event_fd: &EventFd) -> Result<()> {
        self.device
            .enable_irq(VFIO_PCI_INTX_IRQ_INDEX, vec![event_fd])?;

        Ok(())
    }

    pub(crate) fn set_intx_resample_fd(&self, event_fd: &EventFd) -> Result<()> {
        self.device
            .set_irq_resample_fd(VFIO_PCI_INTX_IRQ_INDEX, vec![event_fd])?;

        Ok(())
    }

    pub(crate) fn get_msix_irq_info(&self) -> Option<&VfioIrq> {
        self.device.get_irq_info(VFIO_PCI_MSIX_IRQ_INDEX)
    }

    pub(crate) fn get_msi_irq_info(&self) -> Option<&VfioIrq> {
        self.device.get_irq_info(VFIO_PCI_MSI_IRQ_INDEX)
    }
}
