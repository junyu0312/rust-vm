use std::sync::Arc;

use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;

use crate::arch::Arch;
use crate::arch::irq::InterruptController;
use crate::arch::vcpu::Vcpu;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::error::Result;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub trait Virt: Sized {
    type Arch: Arch;
    type Vcpu: Vcpu<Self::Arch>;
    type Memory: MemoryContainer;

    fn new(num_vcpus: usize) -> Result<Self>;

    fn init_irq(&mut self) -> Result<Arc<dyn InterruptController>>;
    fn init_memory(
        &mut self,
        memory: &mut MemoryAddressSpace<Self::Memory>,
        memory_size: usize,
    ) -> Result<()>;

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout;
    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout;

    fn get_vcpu_number(&self) -> usize;
    fn get_vcpu_mut(&mut self, vcpu: u64) -> Result<Option<&mut Self::Vcpu>>;
    fn get_vcpus(&self) -> Result<&Vec<Self::Vcpu>>;
    fn get_vcpus_mut(&mut self) -> Result<&mut Vec<Self::Vcpu>>;

    fn run(&mut self, device_manager: Arc<dyn DeviceVmExitHandler>) -> Result<()>;
}
