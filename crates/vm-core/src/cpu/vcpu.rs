use std::sync::Arc;

use crate::cpu::vm_exit::VmExit;

pub struct Vcpu {
    pub vcpu_instance: Box<dyn crate::virt::vcpu::Vcpu>,
    pub vm_exit_handler: Arc<dyn VmExit>,
}
