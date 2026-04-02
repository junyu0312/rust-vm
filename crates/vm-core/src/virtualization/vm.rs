use std::sync::Arc;

use thiserror::Error;

use crate::arch::irq::InterruptController;
use crate::virtualization::vcpu::HypervisorVcpu;

pub enum SetUserMemoryRegionFlags {
    ReadWriteExec,
}

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Failed to create irq_chip: {0}")]
    CreateIrqChipError(String),

    #[error("Failed to set_user_memory_region: {0}")]
    SetUserMemoryRegionError(String),

    #[cfg(feature = "hvp")]
    #[error("Applevisor error: {0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),
}

pub trait HypervisorVm: Send + Sync {
    fn create_vcpu(&self, vcpu_id: usize) -> Result<Box<dyn HypervisorVcpu>, VmError>;

    fn create_irq_chip(&self) -> Result<Arc<dyn InterruptController>, VmError>;

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<(), VmError>;
}
