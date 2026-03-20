use std::slice::Iter;

use crate::device::mmio::layout::MmioLayout;
use crate::device::mmio::mmio_as_manager::MmioAddressSpaceManager;
use crate::device::mmio::mmio_device::MmioDevice;
use crate::device::pio::pio_as_manager::PioAddressSpaceManager;
use crate::device::pio::pio_device::PioDevice;
use crate::device_manager::vm_exit::DeviceError;
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::utils::address_space::AddressSpaceError;

pub struct DeviceManager {
    pio_manager: PioAddressSpaceManager,
    mmio_manager: MmioAddressSpaceManager,
}

impl DeviceVmExitHandler for DeviceManager {
    fn io_in(&mut self, port: u16, data: &mut [u8]) -> Result<(), DeviceError> {
        let device = self
            .pio_manager
            .get_device_by_port(port)
            .ok_or(DeviceError::NoDeviceForPort(port))?;

        device.io_in(port, data);

        Ok(())
    }

    fn io_out(&mut self, port: u16, data: &[u8]) -> Result<(), DeviceError> {
        let device = self
            .pio_manager
            .get_device_by_port(port)
            .ok_or(DeviceError::NoDeviceForPort(port))?;

        device.io_out(port, data);

        Ok(())
    }

    fn mmio_read(&self, addr: u64, len: usize, data: &mut [u8]) -> Result<(), DeviceError> {
        let (range, handler) = self
            .mmio_manager
            .get_handler_by_addr(addr)
            .ok_or(DeviceError::NoDeviceForAddr(addr))?;

        let err = || DeviceError::MmioOutOfMemory {
            mmio_start: range.start,
            mmio_len: range.len,
            addr,
        };

        if addr.checked_add(len as u64).ok_or_else(err)?
            > range.start.checked_add(range.len as u64).ok_or_else(err)?
        {
            return Err(err());
        }

        handler.mmio_read(addr - range.start, len, data);

        Ok(())
    }

    fn mmio_write(&self, addr: u64, len: usize, data: &[u8]) -> Result<(), DeviceError> {
        let (range, handler) = self
            .mmio_manager
            .get_handler_by_addr(addr)
            .ok_or(DeviceError::NoDeviceForAddr(addr))?;

        let err = || DeviceError::MmioOutOfMemory {
            mmio_start: range.start,
            mmio_len: range.len,
            addr,
        };

        if addr.checked_add(len as u64).ok_or_else(err)?
            > range.start.checked_add(range.len as u64).ok_or_else(err)?
        {
            return Err(err());
        }

        handler.mmio_write(addr - range.start, len, data);

        Ok(())
    }

    fn in_mmio_region(&self, addr: u64) -> bool {
        self.mmio_manager.mmio_layout().in_mmio_region(addr)
    }
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
