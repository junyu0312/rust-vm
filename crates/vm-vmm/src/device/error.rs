use thiserror::Error;
use vm_core::utils::address_space::AddressSpaceError;

#[derive(Error, Debug)]
pub enum InitDeviceError {
    #[error("Device address space error: {0}")]
    DeviceAddressSpace(#[from] AddressSpaceError),

    #[error("Pci device error: {0}")]
    PciDevice(#[from] vm_pci::error::Error),

    #[error("Failed to register monitor command for device {device}")]
    RegisterMonitorCommand { device: String },
}
