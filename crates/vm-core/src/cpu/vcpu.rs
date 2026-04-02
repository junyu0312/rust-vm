use std::sync::Arc;

use crate::cpu::vm_exit::VmExit;
use crate::hypervisor::vcpu::HypervisorVcpu;

pub struct Vcpu {
    pub vcpu_instance: Box<dyn HypervisorVcpu>,
    pub vm_exit_handler: Arc<dyn VmExit>,
}
