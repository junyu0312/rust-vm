use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use tokio::fs::read;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::firmware::psci::psci_0_2::Psci02;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_core::cpu::vcpu_manager::snapshot::VcpuManagerSnapshot;
use vm_core::virtualization::hypervisor::Hypervisor;
use vm_core::virtualization::vm::SetUserMemoryRegionFlags;
use vm_device::device::Device;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::manager::snapshot::MemoryAddressSpaceSnapshot;

use crate::device::device_manager_v2::snapshot::DeviceSnapshot;
use crate::service::gdbstub::connection::VmGdbStubConnector;
use crate::service::monitor::builder::MonitorServerBuilder;
use crate::vm::Vm;
use crate::vm::VmState;
use crate::vm::config::VmConfig;
use crate::vm::device_builder::DeviceManagerBuilder;
use crate::vm::vm_exit_handler::VmExitHandler;
use crate::vmm::error::VmSnapshotError;
use crate::vmm::error::VmmError;
use crate::vmm::handler::VmmCommand;

#[derive(Serialize, Deserialize)]
pub struct VmSnapshot {
    vm_config: VmConfig,
    vm_state: VmState,
    memory_address_space: MemoryAddressSpaceSnapshot,
    vcpus: VcpuManagerSnapshot,
    irq_chip: Vec<u8>,
    devices: DeviceSnapshot,
}

impl Vm {
    pub async fn build_snapshot(&self) -> Result<VmSnapshot, VmSnapshotError> {
        let vcpus = {
            let vcpu_manager = self.vcpu_manager();
            let mut vcpu_manager = vcpu_manager.lock().await;

            vcpu_manager.build_snapshot().await?
        };

        let irq_chip = {
            let mut buf = vec![];
            self.irq_chip.save(&mut buf)?;
            buf
        };

        let snap = VmSnapshot {
            vm_config: self.vm_config.clone(),
            vm_state: self.vm_state,
            memory_address_space: self.memory_address_space().build_snapshot()?,
            vcpus,
            irq_chip,
            devices: self.device_manager.build_snapshot()?,
        };

        Ok(snap)
    }

    pub async fn from_snapshot(
        hypervisor: &dyn Hypervisor,
        vmm_tx: Arc<mpsc::Sender<VmmCommand>>,
        path: &Path,
    ) -> Result<Self, VmmError> {
        let snap = {
            let buf = read(path)
                .await
                .map_err(|err| VmmError::SnapshotError(VmSnapshotError::Io(err)))?;
            postcard::from_bytes::<VmSnapshot>(&buf)
                .map_err(|err| VmmError::SnapshotError(VmSnapshotError::Postcard(err)))?
        };

        let mut monitor_server_builder = MonitorServerBuilder::default();

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

        let irq_chip: Arc<dyn InterruptController> =
            if !snap.vm_config.devices.iter().any(Device::is_irq_chip) {
                let mut irq_chip = vm_instance.create_irq_chip()?;

                irq_chip
                    .load(&mut Cursor::new(snap.irq_chip))
                    .map_err(|err| VmmError::SnapshotError(VmSnapshotError::IrqChip(err)))?;

                Arc::from(irq_chip)
            } else {
                todo!()
            };

        let device_manager = {
            let mut device_manager = DeviceManagerBuilder::new(
                vm_instance.clone(),
                irq_chip.clone(),
                vm_instance.create_irq_manager()?,
                memory_address_space.clone(),
                &mut monitor_server_builder,
            )?
            .build(&snap.vm_config.devices)?;
            device_manager
                .install_snapshot(snap.devices)
                .map_err(|err| VmmError::SnapshotError(VmSnapshotError::Device(err)))?;

            Arc::new(device_manager)
        };

        let vcpu_manager = Arc::new(Mutex::new(VcpuManager::new(vm_instance.clone())));

        #[cfg(target_arch = "aarch64")]
        let psci = Psci02 {
            vcpu_manager: vcpu_manager.clone(),
        };

        let vm_exit_handler = Arc::new(VmExitHandler::new(
            device_manager.clone(),
            #[cfg(target_arch = "aarch64")]
            psci,
        ));

        {
            let mut vcpu_manager = vcpu_manager.lock().await;

            vcpu_manager
                .install_snapshot(
                    memory_address_space.clone(),
                    vm_exit_handler.clone(),
                    snap.vcpus,
                )
                .await?;
        }

        let gdb_stub = snap
            .vm_config
            .gdb_port
            .map(|port| VmGdbStubConnector::new(vmm_tx, port));

        let vm = Vm {
            vm_config: snap.vm_config,
            _vm_instance: vm_instance,
            vm_state: snap.vm_state,
            vcpu_manager,
            memory_address_space,
            irq_chip,
            device_manager,
            gdb_stub,
            monitor_handlers: monitor_server_builder.components,
        };

        Ok(vm)
    }
}
