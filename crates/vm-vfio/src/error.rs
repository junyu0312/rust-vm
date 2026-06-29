use std::io;

use thiserror::Error;
use vfio_ioctls::VfioError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vfio is supported only on Linux")]
    NotSupport,

    #[error("Failed to map vfio dma, err: {0}")]
    VfioDmaMap(VfioError),

    #[error("Region {0} does not exists")]
    RegionNotExists(usize),

    #[error("Unknown pci header type")]
    UnknownPciHeaderType,

    #[error("Pci device is not endpoint")]
    VfioPciDeviceIsNotEndpoint,

    #[error("Failed to parse msi-x cap")]
    ParseMsiX,

    #[error("Failed to parse msi cap")]
    ParseMsi,

    #[error("Failed to parse intx cap")]
    ParseIntx,

    #[error("Failed to prepare irq, err: {0}")]
    PrepareIrq(Box<dyn std::error::Error + Send + Sync>),

    #[error("Pci bar({0}) has invalid type({1:x}")]
    InvalidMmioBarType(usize, u32),

    #[error("Failed to get info for 64-bit mmio bar({0})")]
    Invalid64BitMemoryBar(usize),

    #[error("Failed to alloc pio region, length: {0}")]
    AllocPio(usize),

    #[error("Failed to alloc irq")]
    AllocIrq,

    #[error("{0}")]
    Vfio(#[from] vfio_ioctls::VfioError),

    #[error("{0}")]
    Pci(#[from] vm_pci::error::Error),

    #[error("Io error: {0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
