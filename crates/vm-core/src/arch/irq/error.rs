use thiserror::Error;

#[derive(Error, Debug)]
pub enum IrqChipError {
    #[error("failed to write device_tree, err:{0}")]
    WriteDeviceTree(String),

    #[error("failed to write device_tree, err: {0}")]
    FdtError(#[from] vm_fdt::Error),

    #[error("failed to build snapshot")]
    SaveSnapshot,

    #[error("failed to load snapshot, err: {0}")]
    LoadSnapshot(Box<dyn std::error::Error + Send + Sync>),
}
