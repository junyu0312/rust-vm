use std::sync::Arc;
use std::sync::Mutex;

use crate::arch::Arch;
use crate::arch::irq::InterruptController;
use crate::arch::vcpu::Vcpu;
use crate::device::mmio::MmioLayout;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::mm::allocator::MemoryContainer;
use crate::mm::manager::MemoryAddressSpace;
use crate::virt::error::VirtError;

pub mod error;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub trait Virt: Sized {
    type Arch: Arch;
    type Vcpu: Vcpu<Self::Arch>;
    type Memory: MemoryContainer;

    fn new(num_vcpus: usize) -> Result<Self, VirtError>;

    fn init_irq(&mut self) -> anyhow::Result<Arc<dyn InterruptController>>;
    fn init_memory(
        &mut self,
        mmio_layout: &MmioLayout,
        memory: &mut MemoryAddressSpace<Self::Memory>,
        memory_size: u64,
    ) -> anyhow::Result<()>;
    fn post_init(&mut self) -> anyhow::Result<()>;

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout;
    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout;

    fn get_vcpu_number(&self) -> usize;
    fn get_vcpu_mut(&mut self, vcpu: u64) -> anyhow::Result<Option<&mut Self::Vcpu>>;
    fn get_vcpus(&self) -> anyhow::Result<&Vec<Self::Vcpu>>;
    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<Self::Vcpu>>;

    fn run(&mut self, device_manager: Arc<Mutex<dyn DeviceVmExitHandler>>) -> anyhow::Result<()>;
}
