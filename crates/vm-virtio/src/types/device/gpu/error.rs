use thiserror::Error;

use crate::types::device::gpu::request::VirtioGpuCtrlType;

#[derive(Error, Debug)]
pub enum VirtioGpuError {
    #[error("unknown ctrl type {0}")]
    UnknownCtrlType(u32),

    #[error("invalid command {0:?}")]
    InvalidCommand(VirtioGpuCtrlType),
}
