use std::sync::Arc;

use vm_mm::manager::MemoryAddressSpace;

use crate::arch::irq::InterruptController;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vm::error::VmError;

pub mod error;

pub enum SetUserMemoryRegionFlags {
    ReadWriteExec,
}

pub trait HypervisorVm: Send + Sync {
    fn create_vcpu(
        &self,
        vcpu_id: usize,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Box<dyn HypervisorVcpu>, VmError>;

    fn create_irq_chip(&self) -> Result<Arc<dyn InterruptController>, VmError>;

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<(), VmError>;
}
