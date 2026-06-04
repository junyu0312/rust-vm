use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid memory format({0})")]
    InvalidMemoryFmt(String),

    #[error("memory too large")]
    MemoryTooLarge(String),
}
