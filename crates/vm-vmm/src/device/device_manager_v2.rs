use std::slice::Iter;
use std::slice::IterMut;

use rangemap::RangeMap;
use tracing::trace;
use vm_core::cpu::vm_exit::VmExitHandlerError;
use vm_core::device::Device;

use crate::device::error::InitDeviceError;

pub(crate) mod snapshot;

#[derive(Default)]
pub struct DeviceManagerV2 {
    devices: Vec<Box<dyn Device>>,

    #[cfg(target_arch = "x86_64")]
    pio_dispatcher: RangeMap<u16, usize>,
    mmio_dispatcher: RangeMap<u64, usize>,
}

impl DeviceManagerV2 {
    pub fn attach_device(&mut self, mut device: Box<dyn Device>) -> Result<(), InitDeviceError> {
        let index = self.devices.len();

        #[cfg(target_arch = "x86_64")]
        if let Some(dev) = device.support_pio_transport_mut() {
            for range in dev.ports() {
                self.pio_dispatcher.insert(range.clone(), index);
            }
        }

        if let Some(dev) = device.support_mmio_transport_mut() {
            for range in dev.mmio_ranges() {
                self.mmio_dispatcher.insert(range.clone(), index);
            }
        }

        self.devices.push(device);

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    pub fn io_in(&self, port: u16, data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        trace!(port, "io in");

        let Some(index) = self.pio_dispatcher.get(&port) else {
            return Err(VmExitHandlerError::NoDeviceForPort(port));
        };

        let device = self.devices.get(*index).unwrap();

        let pio_device = device.support_pio_transport().unwrap();
        pio_device.io_in(port, data)?;

        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn io_in(&self, port: u16, _data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        Err(VmExitHandlerError::NoDeviceForPort(port))
    }

    #[cfg(target_arch = "x86_64")]
    pub fn io_out(&self, port: u16, data: &[u8]) -> Result<(), VmExitHandlerError> {
        trace!(port, ?data, "io out");

        let Some(index) = self.pio_dispatcher.get(&port) else {
            return Err(VmExitHandlerError::NoDeviceForPort(port));
        };

        let device = self.devices.get(*index).unwrap();

        let pio_device = device.support_pio_transport().unwrap();
        pio_device.io_out(port, data)?;

        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn io_out(&self, port: u16, _data: &[u8]) -> Result<(), VmExitHandlerError> {
        Err(VmExitHandlerError::NoDeviceForPort(port))
    }

    pub fn mmio_read(&self, addr: u64, data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        trace!(addr, "mmio read");

        let Some(index) = self.mmio_dispatcher.get(&addr) else {
            return Err(VmExitHandlerError::NoDeviceForAddr(addr));
        };

        let device = self.devices.get(*index).unwrap();

        let mmio_device = device.support_mmio_transport().unwrap();
        mmio_device.mmio_read(addr, data)?;

        Ok(())
    }

    pub fn mmio_write(&self, addr: u64, data: &[u8]) -> Result<(), VmExitHandlerError> {
        trace!(addr, "mmio write");

        let Some(index) = self.mmio_dispatcher.get(&addr) else {
            return Err(VmExitHandlerError::NoDeviceForAddr(addr));
        };

        let device = self.devices.get(*index).unwrap();

        let mmio_device = device.support_mmio_transport().unwrap();
        mmio_device.mmio_write(addr, data)?;

        Ok(())
    }

    pub fn iter(&self) -> Iter<'_, Box<dyn Device>> {
        self.devices.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn Device>> {
        self.devices.iter_mut()
    }
}
