use std::path::Path;

use vm_vfio::vfio::container::VfioContainer;
use vm_vfio::vfio::device::VfioDevice;
use vm_vfio::vfio_pci::device::VfioPciDevice;

use crate::device::error::InitDeviceError;
use crate::vm::device_builder::DeviceManagerBuilder;

impl<'a> DeviceManagerBuilder<'a> {
    pub fn init_vfio(&mut self) -> Result<(), InitDeviceError> {
        let vfio_container = VfioContainer::new()?;

        self.vfio_container
            .set(vfio_container)
            .map_err(|_| InitDeviceError::VfioAlreadtInit)?;

        Ok(())
    }

    pub fn init_vfio_device(
        &mut self,
        name: String,
        path: &Path,
    ) -> Result<VfioPciDevice, InitDeviceError> {
        let container = self
            .vfio_container
            .get()
            .ok_or(InitDeviceError::VfioContainerNotInit)?;

        let vfio_device = VfioDevice::new(path, container)?;

        let vfio_pci_device = VfioPciDevice::new(
            name,
            self.vm.clone(),
            #[cfg(target_arch = "x86_64")]
            self.pci_pio_allocator.get_mut().unwrap(),
            self.pci_mmio_allocator.get_mut().unwrap(),
            self.interrupt_manager.clone(),
            vfio_device,
        )?;

        Ok(vfio_pci_device)
    }

    pub fn vfio_dma_map(&mut self) -> Result<(), InitDeviceError> {
        if !self.need_dma_map {
            return Ok(());
        }

        for region in self.memory.regions().values() {
            let container = self
                .vfio_container
                .get()
                .ok_or(InitDeviceError::VfioContainerNotInit)?;

            unsafe {
                container.vfio_dma_map(region.gpa, region.len(), region.hva())?;
            };
        }

        Ok(())
    }
}
