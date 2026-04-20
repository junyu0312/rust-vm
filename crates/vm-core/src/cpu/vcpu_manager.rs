use std::sync::Arc;

use tokio::sync::Mutex;

use crate::cpu::vcpu::Vcpu;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vm::HypervisorVm;
use crate::virtualization::vm::VmError;

pub struct VcpuManager {
    vm_instance: Arc<dyn HypervisorVm>,
    vcpus: Vec<Arc<Mutex<Vcpu>>>,
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

    pub fn get_vcpu(&self, vcpu_id: usize) -> Option<Arc<Mutex<Vcpu>>> {
        self.vcpus.get(vcpu_id).cloned()
    }

    pub fn create_vcpu(
        &mut self,
        vcpu_id: usize,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<(), VmError> {
        let vcpu_instance = self.vm_instance.create_vcpu(vcpu_id, vm_exit_handler)?;

        let vcpu = Vcpu {
            vcpu_id,
            vcpu_instance,
            booted: false,
        };

        self.vcpus.push(Arc::new(Mutex::new(vcpu)));

        Ok(())
    }

    pub async fn pause_all_vcpus(&self) -> Result<(), VmError> {
        for vcpu in &self.vcpus {
            let mut vcpu = vcpu.lock().await;

            vcpu.pause().await?;
        }

        Ok(())
    }

    pub async fn resume_all_vcpus(&self) -> Result<(), VmError> {
        for vcpu in &self.vcpus {
            let mut vcpu = vcpu.lock().await;

            vcpu.resume().await?;
        }

        Ok(())
    }
}
