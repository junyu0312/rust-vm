use std::error::Error;

use thiserror::Error;
use vm_core::device::error::DeviceError;
use vm_core::utils::address_space::AddressSpaceError;

#[derive(Error, Debug)]
pub enum InitDeviceError {
    #[error("Device manager not set")]
    DeviceManagerNotSet,

    #[error("Device address space error: {0}")]
    DeviceAddressSpace(#[from] AddressSpaceError),

    #[error("Device err: {0}")]
    Device(#[from] DeviceError),

    #[error("Pci device error: {0}")]
    PciDevice(#[from] vm_pci::error::Error),

    #[error("Failed to register pci device")]
    RegisterPciDevice,

    #[error("Failed to alloc resource, {0}")]
    AllocResource(Box<dyn Error + Send + Sync>),

    #[error("Failed to register monitor command for device {device}")]
    RegisterMonitorCommand { device: String },

    #[error("Vfio not support")]
    VfioNotSupport,

    #[error("Vfio already init")]
    VfioAlreadtInit,

    #[error("Vfio container does not init")]
    VfioContainerNotInit,

    #[cfg(target_os = "linux")]
    #[error("Vfio error: {0} ")]
    Vfio(#[from] vm_vfio::error::Error),
}
