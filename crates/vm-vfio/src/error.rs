use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Vfio(#[from] vfio_ioctls::VfioError),
}

pub type Result<T> = std::result::Result<T, Error>;
