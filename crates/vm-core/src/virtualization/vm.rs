use std::sync::Arc;

use vm_mm::manager::MemoryAddressSpace;
#[cfg(target_os = "linux")]
use vmm_sys_util::eventfd::EventFd;

use crate::arch::irq::InterruptController;
use crate::cpu::vm_exit::VmExit;
use crate::interrupt_manager::InterruptManager;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vm::error::VmError;

pub mod error;
pub mod state;

pub enum SetUserMemoryRegionFlags {
    ReadWriteExec,
}

pub trait HypervisorVm: Send + Sync {
    fn create_vcpu(
        &self,
        vcpu_id: u64,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Box<dyn HypervisorVcpu>, VmError>;

    fn create_irq_chip(&self) -> Result<Box<dyn InterruptController>, VmError>;

    fn create_irq_manager(&self) -> Result<InterruptManager, VmError>;

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<(), VmError>;

    #[cfg(target_os = "linux")]
    fn set_irqfd(&self, fd: &EventFd, gsi: u32) -> Result<(), VmError>;

    #[cfg(target_os = "linux")]
    fn del_irqfd(&self, fd: &EventFd, gsi: u32) -> Result<(), VmError>;

    #[cfg(target_os = "linux")]
    fn set_irqfd_with_resample(
        &self,
        fd: &EventFd,
        resamplefd: &EventFd,
        gsi: u32,
    ) -> Result<(), VmError>;

    #[cfg(target_os = "linux")]
    fn set_gsi_routing(&self) -> Result<(), VmError>;
}
