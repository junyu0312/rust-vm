use std::sync::Arc;

use vm_mm::manager::MemoryAddressSpace;

use crate::arch::Arch;
use crate::arch::irq::InterruptController;
use crate::arch::vcpu::Vcpu;
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::error::Result;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub trait Virt: Sized {
    type Arch: Arch;
    type Vcpu: Vcpu<Self::Arch>;

    fn new(num_vcpus: usize) -> Result<Self>;

    fn init_irq(&mut self) -> Result<Arc<dyn InterruptController>>;
    fn init_memory(&mut self, memory: &mut MemoryAddressSpace, memory_size: usize) -> Result<()>;

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout;
    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout;

    fn get_vcpu_number(&self) -> usize;

    fn run(&mut self, device_vm_exit_handler: &dyn DeviceVmExitHandler) -> Result<()>;
}
