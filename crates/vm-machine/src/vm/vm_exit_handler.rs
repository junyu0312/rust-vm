use std::sync::Arc;

#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::Psci;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::psci_0_2::Psci02;
use vm_core::device_manager::manager::DeviceManager;
use vm_core::vcpu::vm_exit::VmExit;
use vm_core::vcpu::vm_exit::VmExitHandlerError;

pub struct VmExitHandler {
    pub device_manager: Arc<DeviceManager>,
    #[cfg(target_arch = "aarch64")]
    pub psci: Psci02,
}

impl VmExit for VmExitHandler {
    fn io_in(&mut self, port: u16, data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        let device = self
            .device_manager
            .pio_manager
            .get_device_by_port(port)
            .ok_or(VmExitHandlerError::NoDeviceForPort(port))?;

        device.io_in(port, data);

        Ok(())
    }

    fn io_out(&mut self, port: u16, data: &[u8]) -> Result<(), VmExitHandlerError> {
        let device = self
            .device_manager
            .pio_manager
            .get_device_by_port(port)
            .ok_or(VmExitHandlerError::NoDeviceForPort(port))?;

        device.io_out(port, data);

        Ok(())
    }

    fn mmio_read(&self, addr: u64, len: usize, data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        let (range, handler) = self
            .device_manager
            .mmio_manager
            .get_handler_by_addr(addr)
            .ok_or(VmExitHandlerError::NoDeviceForAddr(addr))?;

        let err = || VmExitHandlerError::MmioOutOfMemory {
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

    fn mmio_write(&self, addr: u64, len: usize, data: &[u8]) -> Result<(), VmExitHandlerError> {
        let (range, handler) = self
            .device_manager
            .mmio_manager
            .get_handler_by_addr(addr)
            .ok_or(VmExitHandlerError::NoDeviceForAddr(addr))?;

        let err = || VmExitHandlerError::MmioOutOfMemory {
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
        self.device_manager
            .mmio_manager
            .mmio_layout()
            .in_mmio_region(addr)
    }

    #[cfg(target_arch = "aarch64")]
    fn call_smc(&self, vcpu: &mut dyn vm_core::virt::vcpu::Vcpu) -> Result<(), VmExitHandlerError> {
        self.psci.call(vcpu)?;

        Ok(())
    }
}
