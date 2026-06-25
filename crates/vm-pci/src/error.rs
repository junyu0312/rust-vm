use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no space to allocate capability")]
    CapNoSpace,

    #[error("the capability is too large")]
    CapTooLarge,

    #[error("failed to register pci device")]
    FailedRegisterPciDevice,

    #[error("Failed to alloc pio from pio window")]
    AllocPio,

    #[error("Failed to alloc mmio from mmio window")]
    AllocMmio,
}
