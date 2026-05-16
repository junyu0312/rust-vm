use serde::Deserialize;
use serde::Serialize;
use vm_mm::manager::snapshot::MemoryAddressSpaceSnapshot;

use crate::vm::Vm;
use crate::vm::config::VmConfig;
use crate::vmm::error::VmSnapshotError;

#[derive(Serialize, Deserialize)]
pub struct VmSnapshot {
    vm_config: VmConfig,
    memory_address_space: MemoryAddressSpaceSnapshot,
}

impl Vm {
    pub fn build_snapshot(&self) -> Result<VmSnapshot, VmSnapshotError> {
        let snap = VmSnapshot {
            vm_config: self.vm_config.clone(),
            memory_address_space: self.memory_address_space().build_snapshot()?,
        };

        Ok(snap)
    }
}
