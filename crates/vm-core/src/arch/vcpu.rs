use crate::arch::Arch;
use crate::device::mmio::MmioLayout;
use crate::error::Result;

pub trait Vcpu<A>
where
    A: Arch,
{
    fn run(&mut self, mmio_layout: &MmioLayout) -> Result<A::VmExitReason>;
}
