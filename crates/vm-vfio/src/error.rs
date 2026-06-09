use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vfio is supported only on Linux")]
    NotSupport,

    #[error("No pci config region")]
    NoPciConfigRegion,

    #[error("Failed to process vfio pci header")]
    PciHeader,

    #[error("{0}")]
    Vfio(#[from] vfio_ioctls::VfioError),

    #[error("{0}")]
    Pci(#[from] vm_pci::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
