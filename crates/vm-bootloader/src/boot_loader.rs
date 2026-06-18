use std::path::PathBuf;
use std::slice::Iter;

use async_trait::async_trait;
use thiserror::Error;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::error::CpuError;
use vm_core::cpu::vcpu::Vcpu;
use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_firmware::acpi::error::AcpiError;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use vm_utils::range_allocator::RangeAllocatorError;

use crate::initrd_loader::InitrdLoaderError;
use crate::kernel_loader::error::KernelLoaderError;

pub mod arch;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Load dtb failed, reason: {0}")]
    LoadDtbFailed(String),

    #[error("Failed to loader kernel, err: {0}")]
    KernelLoader(#[from] KernelLoaderError),

    #[error("Failed to load initrd, err: {0}")]
    LoadInitrdFailed(#[from] InitrdLoaderError),

    #[error("Memory overlap")]
    MemoryOverlap,

    #[error("Failed to reserve memory, err: {0}")]
    ReserveMemory(#[from] RangeAllocatorError),

    #[error("Setup boot cpu error: {0}")]
    SetupBootCpuError(#[from] CpuError),

    #[error("{0}")]
    GenerateDtb(#[from] vm_fdt::Error),

    #[error("Vcpu too much")]
    VcpuExceedsAcpiCapability,

    #[error("Failed to setup acpi, err: {0}")]
    Acpi(#[from] AcpiError),

    #[error("Device error: {0}")]
    DeviceError(#[from] DeviceError),
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait BootLoaderBuilder
where
    Self: BootLoader,
{
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self;
}

#[async_trait]
pub trait BootLoader {
    #[allow(clippy::too_many_arguments)]
    async fn load(
        &self,
        ram_size: u64,
        vcpus: usize,
        boot_vcpu: &mut Vcpu,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<()>;
}
