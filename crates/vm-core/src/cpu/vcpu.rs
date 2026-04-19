use crate::cpu::error::VcpuError;
use crate::virtualization::vcpu::HypervisorVcpu;

pub struct Vcpu {
    pub vcpu_id: usize,
    pub vcpu_instance: Box<dyn HypervisorVcpu>,
}

impl Vcpu {
    pub fn get_registers(&self) -> Result<(), VcpuError> {
        todo!()
    }

    pub fn write_registers(&self) -> Result<(), VcpuError> {
        todo!()
    }
}
