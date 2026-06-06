use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vfio is supported only on Linux")]
    NotSupport,

    #[error("{0}")]
    Vfio(#[from] vfio_ioctls::VfioError),

    #[error("{0}")]
    Pci(#[from] vm_pci::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
