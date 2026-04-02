use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no space to allocate capability")]
    CapNoSpace,
    #[error("the capability is too large")]
    CapTooLarge,
    #[error("failed to register pci device")]
    FailedRegisterPciDevice,
}
