use std::sync::Arc;

use crate::arch::irq::InterruptController;
use crate::error::Error;
use crate::virt::vcpu::Vcpu;

pub enum SetUserMemoryRegionFlags {
    ReadWriteExec,
}

pub trait Vm: Send + Sync {
    fn create_vcpu(&self, vcpu_id: usize) -> Result<Box<dyn Vcpu>, Error>;

    fn create_irq_chip(&self) -> Result<Arc<dyn InterruptController>, Error>;

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<(), Error>;
}
