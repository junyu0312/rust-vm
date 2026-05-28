use thiserror::Error;
use vm_core::arch::irq::error::IrqChipError;
use vm_core::cpu::error::CpuError;
use vm_core::device::error::DeviceSnapshotError;
use vm_core::monitor::MonitorError;
use vm_core::virtualization::hypervisor::error::HypervisorError;
use vm_core::virtualization::vm::error::VmError;

use crate::device::error::InitDeviceError;

#[derive(Error, Debug)]
pub enum VmSnapshotError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Postcard(#[from] postcard::Error),

    #[error("memory error: {0}")]
    Memory(#[from] vm_mm::error::Error),

    #[error("cpu error: {0}")]
    Cpu(#[from] CpuError),

    #[error("device error: {0}")]
    Device(#[from] DeviceSnapshotError),

    #[error("irq chip error: {0}")]
    IrqChip(#[from] IrqChipError),

    #[error("vm error: {0}")]
    Vm(#[from] VmError),
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

    #[error("{0}")]
    Memory(#[from] vm_mm::error::Error),

    #[error("Failed to setup with bootloader, error: {0}")]
    Bootloader(#[from] vm_bootloader::boot_loader::Error),

    #[error("monitor error: {0}")]
    Monitor(#[from] MonitorError),

    #[error("Save vm error: {0}")]
    SnapshotError(#[from] VmSnapshotError),
}
