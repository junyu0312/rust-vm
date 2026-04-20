use std::sync::Arc;

use crate::cpu::error::VcpuError;
use crate::cpu::vcpu::Vcpu;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vm::HypervisorVm;
use crate::virtualization::vm::VmError;

pub struct VcpuManager {
    vm_instance: Arc<dyn HypervisorVm>,
    vcpus: Vec<Vcpu>,
}

impl VcpuManager {
    pub fn new(vm_instance: Arc<dyn HypervisorVm>) -> Self {
        VcpuManager {
            vm_instance,
            vcpus: Default::default(),
        }
    }

    pub fn get_active_vcpus(&self) -> usize {
        self.vcpus.len()
    }

    pub fn get_vcpu(&self, vcpu_id: usize) -> Result<&Vcpu, VcpuError> {
        self.vcpus
            .get(vcpu_id)
            .ok_or(VcpuError::VcpuNotCreated(vcpu_id))
    }

    pub fn get_vcpu_mut(&mut self, vcpu_id: usize) -> Result<&mut Vcpu, VcpuError> {
        self.vcpus
            .get_mut(vcpu_id)
            .ok_or(VcpuError::VcpuNotCreated(vcpu_id))
    }

    pub fn create_vcpu(
        &mut self,
        vcpu_id: usize,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<(), VmError> {
        let vcpu_instance = self.vm_instance.create_vcpu(vcpu_id, vm_exit_handler)?;

        let vcpu = Vcpu::new(vcpu_id, vcpu_instance);

        self.vcpus.push(vcpu);

        Ok(())
    }

    pub async fn pause_all_vcpus(&mut self) -> Result<(), VmError> {
        for vcpu in &mut self.vcpus {
            vcpu.pause().await?;
        }

        Ok(())
    }

    pub async fn resume_all_vcpus(&mut self) -> Result<(), VmError> {
        for vcpu in &mut self.vcpus {
            vcpu.resume().await?;
        }

        Ok(())
    }
}
