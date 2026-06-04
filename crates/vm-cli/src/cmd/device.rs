use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum Device {
    GicV3,
    VirtioMmioBalloon,
    VirtioMmioEntropy,
    VirtioPciEntropy,
    VfioPci { host: String },
}

impl From<Device> for vm_device::device::Device {
    fn from(device: Device) -> Self {
        match device {
            Device::GicV3 => vm_device::device::Device::GicV3,
            Device::VirtioMmioBalloon => vm_device::device::Device::VirtioMmioBalloon,
            Device::VirtioMmioEntropy => vm_device::device::Device::VirtioMmioEntropy,
            Device::VirtioPciEntropy => vm_device::device::Device::VirtioPciEntropy,
            Device::VfioPci { host } => vm_device::device::Device::VfioPci { host },
        }
    }
}
