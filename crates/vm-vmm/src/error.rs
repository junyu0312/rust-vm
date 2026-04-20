use thiserror::Error;
use vm_core::cpu::error::VcpuError;
use vm_core::monitor::MonitorError;
use vm_core::utils::address_space::AddressSpaceError;
use vm_core::virtualization::hypervisor::HypervisorError;
use vm_core::virtualization::vm::VmError;

use crate::service::gdbstub::error::VmGdbStubError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Vm already exists")]
    VmAlreadyExists,

    #[error("Vm not exists")]
    VmNotExists,

    #[error("Hypervisor error: {0}")]
    HypervisorError(#[from] HypervisorError),

    #[error("Vm error: {0}")]
    VmError(#[from] VmError),

    #[error("Vcpu error: {0}")]
    VcpuError(#[from] VcpuError),

    #[error("Gdb error: {0}")]
    GdbError(#[from] VmGdbStubError),

    #[error("Device address space error: {0}")]
    DeviceAddressSpace(#[from] AddressSpaceError),

    #[error("Pci device error: {0}")]
    PciDevice(#[from] vm_pci::error::Error),

    #[error("{0}")]
    Memory(#[from] vm_mm::error::Error),

    #[error("Failed to init memory, error: {0}")]
    InitMemory(String),

    #[error("Failed to setup with bootloader, error: {0}")]
    Bootloader(#[from] vm_bootloader::boot_loader::Error),

    #[error("gdb_stub failed, error: {0}")]
    GdbStub(String),

    #[error("monitor error: {0}")]
    Monitor(#[from] MonitorError),
}

pub type Result<T> = core::result::Result<T, Error>;
