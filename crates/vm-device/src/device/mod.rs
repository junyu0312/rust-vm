#[cfg(target_os = "linux")]
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

pub mod cmos;
pub mod dummy;
pub mod i8042;
pub mod pic;
pub mod post_debug;
pub mod uart8250;
pub mod virtio;

#[cfg(target_arch = "aarch64")]
pub mod pl011;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum VirtioTransport {
    Mmio,
    Pci,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Device {
    GicV3,
    VirtioBlk {
        transport: VirtioTransport,
    },
    VirtioBalloon {
        transport: VirtioTransport,
    },
    VirtioEntropy {
        transport: VirtioTransport,
    },
    #[cfg(target_os = "linux")]
    VfioPci {
        name: String,
        path: PathBuf,
    },
}

impl Device {
    pub fn is_irq_chip(&self) -> bool {
        match self {
            Device::GicV3 => true,
            Device::VirtioBlk { .. }
            | Device::VirtioBalloon { .. }
            | Device::VirtioEntropy { .. } => false,
            #[cfg(target_os = "linux")]
            Device::VfioPci { .. } => false,
        }
    }

    pub fn is_vfio_device(&self) -> bool {
        match self {
            #[cfg(target_os = "linux")]
            Device::VfioPci { .. } => true,
            _ => false,
        }
    }
}
