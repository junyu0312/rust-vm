use crate::device::mmio::MmioLayout;
use crate::virt::vm_exit::VmExitReason;

pub mod arch;

pub trait Vcpu {
    fn run(&mut self, mmio_layout: &MmioLayout) -> anyhow::Result<VmExitReason>;
}
