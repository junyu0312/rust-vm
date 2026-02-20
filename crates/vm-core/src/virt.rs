use std::sync::Arc;

use crate::arch::Arch;
use crate::device::mmio::MmioLayout;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::irq::InterruptController;
use crate::mm::allocator::MemoryContainer;
use crate::mm::manager::MemoryAddressSpace;
use crate::vcpu::Vcpu;
use crate::virt::error::VirtError;

pub mod error;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub trait VirtBuilder: Sized {
    fn new() -> Result<Self, VirtError>;
}

pub trait Virt: VirtBuilder {
    type Arch: Arch;
    type Vcpu: Vcpu<Self::Arch>;
    type Memory: MemoryContainer;

    fn builtin_irq_chip(&mut self) -> anyhow::Result<Option<Arc<dyn InterruptController>>>;
    fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()>;
    fn init_memory(
        &mut self,
        mmio_layout: &MmioLayout,
        memory: &mut MemoryAddressSpace<Self::Memory>,
        memory_size: u64,
    ) -> anyhow::Result<()>;
    fn post_init(&mut self) -> anyhow::Result<()>;

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout;
    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout;

    fn get_vcpu_mut(&mut self, vcpu: u64) -> anyhow::Result<Option<&mut Self::Vcpu>>;
    fn get_vcpus(&self) -> anyhow::Result<&Vec<Self::Vcpu>>;
    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<Self::Vcpu>>;

    fn run(&mut self, device_manager: &mut dyn DeviceVmExitHandler) -> anyhow::Result<()>;
}
