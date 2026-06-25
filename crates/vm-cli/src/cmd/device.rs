#[cfg(target_os = "linux")]
use std::path::PathBuf;

use serde::Deserialize;
use vm_device::device::VirtioTransport;

#[derive(Debug, Clone, Deserialize)]
pub enum Device {
    GicV3,
    VirtioMmioBlk,
    VirtioPciBlk,
    VirtioMmioBalloon,
    VirtioMmioEntropy,
    VirtioPciEntropy,
    #[cfg(target_os = "linux")]
    VfioPci {
        name: String,
        path: PathBuf,
    },
}

impl From<Device> for vm_device::device::Device {
    fn from(device: Device) -> Self {
        match device {
            Device::GicV3 => vm_device::device::Device::GicV3,
            Device::VirtioMmioBlk => vm_device::device::Device::VirtioBlk {
                transport: VirtioTransport::Mmio,
            },
            Device::VirtioPciBlk => vm_device::device::Device::VirtioBlk {
                transport: VirtioTransport::Pci,
            },
            Device::VirtioMmioBalloon => vm_device::device::Device::VirtioBalloon {
                transport: VirtioTransport::Mmio,
            },
            Device::VirtioMmioEntropy => vm_device::device::Device::VirtioEntropy {
                transport: VirtioTransport::Mmio,
            },
            Device::VirtioPciEntropy => vm_device::device::Device::VirtioEntropy {
                transport: VirtioTransport::Pci,
            },
            #[cfg(target_os = "linux")]
            Device::VfioPci { name, path } => vm_device::device::Device::VfioPci { name, path },
        }
    }
}
