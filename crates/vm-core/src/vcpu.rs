use crate::arch::Arch;
use crate::device::vm_exit::DeviceVmExitHandler;

pub mod arch;

pub trait Vcpu<A>
where
    A: Arch,
{
    fn run(&mut self, device_handler: &dyn DeviceVmExitHandler) -> anyhow::Result<A::VmExitReason>;
}
