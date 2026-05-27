use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::registers::ArchRegisters;
use crate::cpu::vcpu_manager::VcpuManager;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vm::error::VmError;

#[derive(Serialize, Deserialize)]
pub struct VcpuSnapshot {
    booted: bool,
    register: ArchRegisters,
}

#[derive(Serialize, Deserialize)]
pub struct VcpuManagerSnapshot {
    vcpus: Vec<VcpuSnapshot>,
}

impl VcpuManager {
    pub async fn build_snapshot(&mut self) -> Result<VcpuManagerSnapshot, VmError> {
        let mut vcpus = Vec::with_capacity(self.get_active_vcpus());

        for vcpu_id in 0..self.get_active_vcpus() {
            let booted = self.get_vcpu(vcpu_id)?.booted();
            let register = self.get_vcpu_mut(vcpu_id)?.read_registers().await?;

            vcpus.push(VcpuSnapshot { booted, register });
        }

        let snap = VcpuManagerSnapshot { vcpus };

        Ok(snap)
    }

    pub async fn install_snapshot(
        &mut self,
        memory_address_space: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
        snap: VcpuManagerSnapshot,
    ) -> Result<(), VmError> {
        for (vcpu_id, vcpu_snap) in snap.vcpus.into_iter().enumerate() {
            self.create_vcpu(
                vcpu_id,
                memory_address_space.clone(),
                vm_exit_handler.clone(),
                vcpu_snap.booted,
            )?;

            let vcpu = self.get_vcpu_mut(vcpu_id)?;
            vcpu.write_registers(vcpu_snap.register).await?;
        }

        Ok(())
    }
}
