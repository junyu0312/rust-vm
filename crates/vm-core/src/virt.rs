use crate::device::pio::IoAddressSpace;
use crate::irq::InterruptController;
use crate::mm::allocator::MemoryContainer;
use crate::mm::manager::MemoryAddressSpace;
use crate::vcpu::Vcpu;

pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub trait Virt: Sized {
    type Vcpu: Vcpu;
    type Memory: MemoryContainer;
    type Irq: InterruptController;

    fn new() -> anyhow::Result<Self>;

    fn init_irq(&mut self) -> anyhow::Result<Self::Irq>;
    fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()>;
    fn init_memory(&mut self, memory: &mut MemoryAddressSpace<Self::Memory>) -> anyhow::Result<()>;
    fn post_init(&mut self) -> anyhow::Result<()>;

    fn get_vcpu_mut(&mut self, vcpu: u64) -> anyhow::Result<Option<&mut Self::Vcpu>>;

    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()>;
}
