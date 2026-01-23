use crate::virt::vm_exit::VmExitReason;

pub mod arch;

pub trait Vcpu {
    fn run(&mut self) -> anyhow::Result<VmExitReason>;
}
