use crate::arch::Arch;
use crate::device::IoAddressSpace;
use crate::irq::InterruptController;
use crate::mm::allocator::MemoryContainer;
use crate::mm::manager::MemoryAddressSpace;
use crate::vcpu::Vcpu;
use crate::virt::error::VirtError;
use crate::virt::vm_exit::HandleVmExitResult;
use crate::virt::vm_exit::VmExitReason;

pub mod error;
pub mod vm_exit;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub trait Virt: Sized {
    type Arch: Arch;
    type Vcpu: Vcpu;
    type Memory: MemoryContainer;
    type Irq: InterruptController;

    fn new() -> Result<Self, VirtError>;

    fn init_irq(&mut self) -> anyhow::Result<Self::Irq>;
    fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()>;
    fn init_memory(&mut self, memory: &mut MemoryAddressSpace<Self::Memory>) -> anyhow::Result<()>;
    fn post_init(&mut self) -> anyhow::Result<()>;

    fn get_vcpu_mut(&mut self, vcpu: u64) -> anyhow::Result<Option<&mut Self::Vcpu>>;
    fn get_vcpus(&self) -> anyhow::Result<&Vec<Self::Vcpu>>;
    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<Self::Vcpu>>;

    fn handle_vm_exit(
        &self,
        exit_reason: VmExitReason,
        device: &mut IoAddressSpace,
    ) -> Result<HandleVmExitResult, vm_exit::Error>;

    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()>;
}
