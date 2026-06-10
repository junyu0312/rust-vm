use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vfio is supported only on Linux")]
    NotSupport,

    #[error("Region {0} does not exists")]
    RegionNotExists(usize),

    #[error("Pci device is not endpoint")]
    VfioPciDeviceIsNotEndpoint,

    #[error("Pci bar({0}) has invalid type({1:x}")]
    InvalidMmioBarType(usize, u32),

    #[error("Failed to get info for 64-bit mmio bar({0})")]
    Invalid64BitMemoryBar(usize),

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
