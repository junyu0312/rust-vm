use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vfio is supported only on Linux")]
    NotSupport,

    #[error("Region {0} does not exists")]
    RegionNotExists(usize),

    #[error("Failed to process vfio pci header")]
    PciHeader,

    #[error("Pci device is not endpoint")]
    VfioPciDeviceIsNotEndpoint,

    // #[error("The length of bar {bar} too large {size}")]
    // BarRegionTooLarge { bar: usize, size: u64 },
    #[error("Failed to alloc pio region, length: {0}")]
    AllocPio(usize),

    #[error("{0}")]
    Vfio(#[from] vfio_ioctls::VfioError),

    #[error("{0}")]
    Pci(#[from] vm_pci::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
