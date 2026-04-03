use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmGdbStubError {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}
