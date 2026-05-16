use thiserror::Error;

use crate::virtualization::vcpu::error::VcpuError;

#[derive(Error, Debug)]
pub enum CpuError {
    #[error("Vcpu {0} already booted")]
    CpuAlreadyBooted(usize),

    #[error("Failed to boot vcpu {0}")]
    BootVcpu(usize),

    #[error("Vcpu command channel disconnected")]
    VcpuCommandDisconnected,

    #[error("Vcpu error: {0}")]
    VcpuError(#[from] VcpuError),
}
