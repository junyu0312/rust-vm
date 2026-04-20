use std::sync::Arc;

use tokio::sync::Mutex;

use crate::cpu::error::VcpuError;
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
        };

        self.vcpus.push(Arc::new(Mutex::new(vcpu)));

        Ok(())
    }

    pub async fn boot_vcpu(
        &mut self,
        vcpu_id: usize,
        pc: u64,
        dtb_or_context_id: u64,
    ) -> Result<(), VcpuError> {
        let vcpu = self
            .vcpus
            .get(vcpu_id)
            .ok_or(VcpuError::VcpuNotCreated(vcpu_id))?;

        let mut vcpu = vcpu.lock().await;

        #[cfg(target_arch = "aarch64")]
        {
            use crate::arch::aarch64::register::AArch64Registers;

            let register = vcpu.vcpu_instance.read_reigsters().await?;
            let registers = AArch64Registers::boot_registers(
                vcpu_id,
                dtb_or_context_id,
                pc,
                register.pstate,
                register.sctlr_el1,
                register.cnthctl_el2,
            );
            vcpu.vcpu_instance.write_registers(registers).await?;
        }

        vcpu.vcpu_instance.resume().await?;

        Ok(())
    }

    pub async fn tick_all_vcpus(&self) -> Result<(), VmError> {
        for vcpu in &self.vcpus {
            vcpu.lock().await.vcpu_instance.pause().await?;
        }

        Ok(())
    }
}
