use thiserror::Error;
use vm_utils::range_allocator::RangeAllocatorError;

#[derive(Error, Debug)]
pub enum DeviceSnapshotError {
    #[error("save device snapshot io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to serde: {0}")]
    Serde(String),

    #[error("failed to deserde: {0}")]
    Deserde(String),

    #[error("device {0} does not support snapshot")]
    DeviceNotSupportSnapshot(String),
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Failed to alloc resource")]
    AllocResource,

    #[error("Failed to alloc resource")]
    AllocResourceErr(#[from] RangeAllocatorError),

    #[error("Mmio range is empty")]
    MmioRangeIsEmpty,

    #[error("Failed to write fdt: {0}")]
    Fdt(#[from] vm_fdt::Error),
}
