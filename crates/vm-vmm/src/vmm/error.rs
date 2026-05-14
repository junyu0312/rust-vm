use thiserror::Error;
use vm_core::cpu::error::CpuError;
use vm_core::monitor::MonitorError;
use vm_core::virtualization::hypervisor::error::HypervisorError;
use vm_core::virtualization::vm::error::VmError;

use crate::device::error::InitDeviceError;
use crate::service::gdbstub::error::VmGdbStubError;

#[derive(Error, Debug)]
pub enum VmSnapshotError {
    #[error("Failed to save vm due to io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to save vm due to serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Failed to save vm due to memory error: {0}")]
    Memory(#[from] vm_mm::error::Error),

    #[error("Failed to save vm due to cpu error: {0}")]
    Cpu(#[from] CpuError),
}

#[derive(Error, Debug)]
pub enum VmmError {
    #[error("Vm already exists")]
    VmAlreadyExists,

    #[error("Vm not exists")]
    VmNotExists,

    #[error("Hypervisor error: {0}")]
    HypervisorError(#[from] HypervisorError),

    #[error("Vm error: {0}")]
    VmError(#[from] VmError),

    #[error("Cpu error: {0}")]
    CpuError(#[from] CpuError),

    #[error("Failed to init device: {0}")]
    InitDevice(#[from] InitDeviceError),

    #[error("Gdb error: {0}")]
    GdbError(#[from] VmGdbStubError),

    #[error("{0}")]
    Memory(#[from] vm_mm::error::Error),

    #[error("Failed to setup with bootloader, error: {0}")]
    Bootloader(#[from] vm_bootloader::boot_loader::Error),

    #[error("monitor error: {0}")]
    Monitor(#[from] MonitorError),
}
