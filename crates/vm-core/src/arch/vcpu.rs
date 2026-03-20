use crate::arch::Arch;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::error::Result;

pub trait Vcpu<A>
where
    A: Arch,
{
    fn run(&mut self, device_vm_exit_handler: &dyn DeviceVmExitHandler) -> Result<A::VmExitReason>;
}
