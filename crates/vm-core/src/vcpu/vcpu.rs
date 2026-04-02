use std::sync::Arc;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::vcpu::AArch64Vcpu as ArchVcpu;
#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::vm_exit::VmExitReason;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::vcpu::X86_64Vcpu as ArchVcpu;
#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::vm_exit::VmExitReason;
use crate::device_manager::vm_exit::DeviceVmExitHandler;
use crate::vcpu::error::VcpuError;

pub trait Vcpu: ArchVcpu + Send {
    fn vm_exit_handler(&self) -> Arc<dyn DeviceVmExitHandler>;

    fn post_init_within_thread(&mut self) -> Result<(), VcpuError>;

    fn run(&mut self) -> Result<VmExitReason, VcpuError>;
}
