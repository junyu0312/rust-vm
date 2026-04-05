use std::sync::Arc;

use crate::cpu::error::VcpuError;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vcpu::HypervisorVcpu;

pub struct Vcpu {
    pub vcpu_id: usize,
    pub vcpu_instance: Box<dyn HypervisorVcpu>,
    pub vm_exit_handler: Arc<dyn VmExit>,
}

impl Vcpu {
    pub fn get_registers(&self) -> Result<(), VcpuError> {
        todo!()
    }

    pub fn write_registers(&self) -> Result<(), VcpuError> {
        todo!()
    }
}
