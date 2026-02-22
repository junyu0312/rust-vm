use crate::arch::Arch;
use crate::device::mmio::MmioLayout;

pub mod arch;

pub trait Vcpu<A>
where
    A: Arch,
{
    fn run(&mut self, mmio_layout: &MmioLayout) -> anyhow::Result<A::VmExitReason>;
}
