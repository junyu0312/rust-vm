use thiserror::Error;

use crate::virtualization::vcpu::error::VcpuError;

#[derive(Error, Debug)]
pub enum CpuError {
    #[error("Vcpu command channel disconnected")]
    VcpuCommandDisconnected,

    #[error("Vcpu error: {0}")]
    VcpuError(#[from] VcpuError),
}
