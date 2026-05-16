use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc;
use vm_core::virtualization::hypervisor::Hypervisor;
use vm_core::virtualization::vm::SetUserMemoryRegionFlags;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::manager::snapshot::MemoryAddressSpaceSnapshot;

use crate::vm::Vm;
use crate::vm::config::VmConfig;
use crate::vmm::error::VmSnapshotError;
use crate::vmm::error::VmmError;
use crate::vmm::handler::VmmCommand;

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

    // TODO
    #[allow(warnings)]
    pub fn from_snapshot(
        hypervisor: &dyn Hypervisor,
        vmm_tx: Arc<mpsc::Sender<VmmCommand>>,
        snap: VmSnapshot,
    ) -> Result<Self, VmmError> {
        let vm_instance = hypervisor.create_vm()?;

        let memory_address_space = {
            let memory_address_space =
                MemoryAddressSpace::from_snapshot(snap.memory_address_space)?;

            for (gpa, memory_region) in memory_address_space.regions() {
                vm_instance.set_user_memory_region(
                    memory_region.hva() as _,
                    *gpa,
                    memory_region.len(),
                    SetUserMemoryRegionFlags::ReadWriteExec,
                )?;
            }

            Arc::new(memory_address_space)
        };

        let vm = Vm {
            vm_config: snap.vm_config,
            _vm_instance: vm_instance,
            vcpu_manager: todo!(),
            memory_address_space,
            _irq_chip: todo!(),
            device_manager: todo!(),
            gdb_stub: todo!(),
            monitor_handlers: todo!(),
        };

        Ok(vm)
    }
}
