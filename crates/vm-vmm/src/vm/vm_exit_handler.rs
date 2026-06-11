use std::sync::Arc;

#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::Psci;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::psci_0_2::Psci02;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::vcpu::AArch64Vcpu;
use vm_core::cpu::vm_exit::VmExit;
use vm_core::cpu::vm_exit::VmExitHandlerError;

use crate::device::device_manager_v2::DeviceManagerV2;

pub struct VmExitHandler {
    device_manager: Arc<DeviceManagerV2>,
    #[cfg(target_arch = "aarch64")]
    psci: Psci02,
}

impl VmExitHandler {
    pub fn new(
        device_manager: Arc<DeviceManagerV2>,
        #[cfg(target_arch = "aarch64")] psci: Psci02,
    ) -> Self {
        VmExitHandler {
            device_manager,
            #[cfg(target_arch = "aarch64")]
            psci,
        }
    }
}

impl VmExit for VmExitHandler {
    fn io_in(&self, port: u16, data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        self.device_manager.io_in(port, data)?;

        Ok(())
    }

    fn io_out(&self, port: u16, data: &[u8]) -> Result<(), VmExitHandlerError> {
        self.device_manager.io_out(port, data)?;

        Ok(())
    }

    fn mmio_read(&self, addr: u64, data: &mut [u8]) -> Result<(), VmExitHandlerError> {
        self.device_manager.mmio_read(addr, data)
    }

    fn mmio_write(&self, addr: u64, data: &[u8]) -> Result<(), VmExitHandlerError> {
        self.device_manager.mmio_write(addr, data)
    }

    #[cfg(target_arch = "aarch64")]
    fn call_smc(&self, vcpu: &mut dyn AArch64Vcpu) -> Result<(), VmExitHandlerError> {
        self.psci.call(vcpu)?;

        Ok(())
    }
}
