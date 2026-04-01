use std::sync::Arc;

use crate::arch::Arch;
use crate::arch::irq::InterruptController;
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::error::Result;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub enum SetUserMemoryRegionFlags {
    ReadWriteExec,
}

pub trait Virt: Sized {
    type Arch: Arch;

    fn new(num_vcpus: usize) -> Result<Self>;

    fn create_irq_chip(&mut self) -> Result<Arc<dyn InterruptController>>;

    fn set_user_memory_region(
        &mut self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<()>;

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout;
    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout;

    fn get_vcpu_number(&self) -> usize;

    fn run(&mut self, device_vm_exit_handler: &dyn DeviceVmExitHandler) -> Result<()>;
}
