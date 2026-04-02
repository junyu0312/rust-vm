use std::sync::Arc;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::firmware::psci::Psci;
use crate::device_manager::vm_exit::DeviceVmExitHandler;

pub struct Vcpu {
    pub vcpu_instance: Box<dyn crate::virt::vcpu::Vcpu>,
    pub device_vm_exit_handler: Arc<dyn DeviceVmExitHandler>,
    #[cfg(target_arch = "aarch64")]
    pub psci: Arc<dyn Psci>,
}
