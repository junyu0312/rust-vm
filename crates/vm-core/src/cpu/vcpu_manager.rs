use std::sync::Arc;

use vm_mm::manager::MemoryAddressSpace;

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

    pub fn get_vcpu(&self, vcpu_id: usize) -> Result<&Vcpu, VmError> {
        self.vcpus
            .get(vcpu_id)
            .ok_or(VmError::VcpuNotCreated(vcpu_id))
    }

    pub fn get_vcpu_mut(&mut self, vcpu_id: usize) -> Result<&mut Vcpu, VmError> {
        self.vcpus
            .get_mut(vcpu_id)
            .ok_or(VmError::VcpuNotCreated(vcpu_id))
    }

    pub fn create_vcpu(
        &mut self,
        vcpu_id: usize,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<(), VmError> {
        let vcpu_instance = self.vm_instance.create_vcpu(vcpu_id, mm, vm_exit_handler)?;

        let vcpu = Vcpu::new(vcpu_instance);

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
