use std::slice::Iter;

use crate::device::mmio::layout::MmioLayout;
use crate::device::mmio::mmio_as_manager::MmioAddressSpaceManager;
use crate::device::mmio::mmio_device::MmioDevice;
use crate::device::pio::pio_as_manager::PioAddressSpaceManager;
use crate::device::pio::pio_device::PioDevice;
use crate::utils::address_space::AddressSpaceError;

pub struct DeviceManager {
    pub pio_manager: PioAddressSpaceManager,
    pub mmio_manager: MmioAddressSpaceManager,
}

impl DeviceManager {
    pub fn new(mmio_layout: MmioLayout) -> Self {
        DeviceManager {
            pio_manager: PioAddressSpaceManager::default(),
            mmio_manager: MmioAddressSpaceManager::new(mmio_layout),
        }
    }

    pub fn register_pio_device(
        &mut self,
        device: Box<dyn PioDevice>,
    ) -> Result<(), AddressSpaceError> {
        self.pio_manager.register(device)
    }

    pub fn register_mmio_device(
        &mut self,
        device: Box<dyn MmioDevice>,
    ) -> Result<(), AddressSpaceError> {
        self.mmio_manager.register(device)
    }

    pub fn mmio_devices(&self) -> Iter<'_, Box<dyn MmioDevice>> {
        self.mmio_manager.devices()
    }
}
