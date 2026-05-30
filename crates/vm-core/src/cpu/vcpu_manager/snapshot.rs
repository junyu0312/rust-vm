use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use vm_mm::manager::MemoryAddressSpace;

use crate::cpu::vcpu_manager::VcpuManager;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::vm::error::VmError;

#[derive(Serialize, Deserialize)]
pub struct VcpuSnapshot {
    booted: bool,
    state: Vec<u8>,
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
            let state = self.get_vcpu(vcpu_id)?.save().await?;

            vcpus.push(VcpuSnapshot { booted, state });
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
                vcpu_id as u64,
                memory_address_space.clone(),
                vm_exit_handler.clone(),
                vcpu_snap.booted,
            )?;

            let vcpu = self.get_vcpu_mut(vcpu_id)?;
            vcpu.load(vcpu_snap.state).await?;
        }

        Ok(())
    }
}
