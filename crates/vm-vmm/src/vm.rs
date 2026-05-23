use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use tempfile::NamedTempFile;
use tokio::sync::Mutex;
use vm_core::arch::irq::InterruptController;
use vm_core::arch::registers::ArchCoreRegisters;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_core::device_manager::DeviceManager;
use vm_core::monitor::MonitorCommandOps;
use vm_core::virtualization::vcpu::error::VcpuError;
use vm_core::virtualization::vm::HypervisorVm;
use vm_core::virtualization::vm::error::VmError;
use vm_core::virtualization::vm::state::VmState;
use vm_mm::manager::MemoryAddressSpace;

use crate::service::gdbstub::connection::VmGdbStubConnector;
use crate::vm::config::VmConfig;
use crate::vmm::error::VmSnapshotError;

pub mod config;

mod snapshot;
mod vm_exit_handler;

const PAGE_SIZE: usize = 4 << 10;

pub struct Vm {
    vm_config: VmConfig,
    _vm_instance: Arc<dyn HypervisorVm>,
    vm_state: VmState,
    vcpu_manager: Arc<Mutex<VcpuManager>>,
    memory_address_space: Arc<MemoryAddressSpace>,
    _irq_chip: Arc<dyn InterruptController>,
    device_manager: Arc<DeviceManager>,
    gdb_stub: Option<VmGdbStubConnector>,
    monitor_handlers: HashMap<String, Box<dyn MonitorCommandOps>>,
}

impl Vm {
    pub fn vcpu_manager(&self) -> Arc<Mutex<VcpuManager>> {
        self.vcpu_manager.clone()
    }

    pub fn memory_address_space(&self) -> &MemoryAddressSpace {
        self.memory_address_space.as_ref()
    }

    pub fn monitor_handlers(&self) -> &HashMap<String, Box<dyn MonitorCommandOps>> {
        &self.monitor_handlers
    }

    pub async fn boot(&mut self) -> Result<(), VmError> {
        self.vm_state.ensure_is_created()?;

        let mut stop_on_boot = false;

        if let Some(gdb_stub) = &self.gdb_stub {
            stop_on_boot = true;
            gdb_stub
                .spawn_listener()
                .map_err(|_| VmError::GdbListenerCreation)?;
        }

        let mut vcpu_manager = self.vcpu_manager.lock().await;

        if !stop_on_boot {
            vcpu_manager.get_vcpu_mut(0)?.boot().await?;

            self.vm_state = VmState::Running;
        }

        Ok(())
    }

    pub async fn read_core_registers(&self, vcpu_id: usize) -> Result<ArchCoreRegisters, VmError> {
        self.vm_state.ensure_is_not_running()?;

        let mut vcpu_manager = self.vcpu_manager.lock().await;
        let vcpu = vcpu_manager.get_vcpu_mut(vcpu_id)?;

        let regs = vcpu.read_core_registers().await?;

        Ok(regs)
    }

    pub async fn write_core_registers(
        &self,
        vcpu_id: usize,
        registers: ArchCoreRegisters,
    ) -> Result<(), VmError> {
        self.vm_state.ensure_is_not_running()?;

        let mut vcpu_manager = self.vcpu_manager.lock().await;
        let vcpu = vcpu_manager.get_vcpu_mut(vcpu_id)?;

        vcpu.write_core_registers(registers).await?;

        Ok(())
    }

    pub async fn read_addrs(
        &self,
        gva: u64,
        len: usize,
        vcpu_id: usize,
    ) -> Result<Vec<u8>, VmError> {
        self.vm_state.ensure_is_not_running()?;

        let vcpu_manager = self.vcpu_manager();
        let mut vcpu_manager = vcpu_manager.lock().await;
        let vcpu = vcpu_manager.get_vcpu_mut(vcpu_id)?;

        let mut len = len;
        let mut buf = Vec::with_capacity(len);
        while len > 0 {
            let Some(gpa) = vcpu.translate_gva_to_gpa(gva).await? else {
                return Err(VmError::CpuError(VcpuError::TranslateErr.into()));
            };

            let hva = self.memory_address_space().gpa_to_hva(gpa).unwrap();
            buf.push(unsafe { *hva });
            // TODO: Opt
            len -= 1;
        }

        Ok(buf)
    }

    pub async fn write_addrs(
        &mut self,
        _gpa: u64,
        _data: &[u8],
        vcpu_id: usize,
    ) -> Result<(), VmError> {
        self.vm_state.ensure_is_not_running()?;

        let vcpu_manager = self.vcpu_manager();
        let mut vcpu_manager = vcpu_manager.lock().await;
        let _vcpu = vcpu_manager.get_vcpu_mut(vcpu_id)?;

        let _buf = todo!();
    }

    pub async fn get_active_vcpus(&self) -> usize {
        // TODO: is it necessary?
        // self.vm_state.ensure_is_not_running()?;

        let vcpu_manager = self.vcpu_manager();
        let vcpu_manager = vcpu_manager.lock().await;

        vcpu_manager.get_active_vcpus()
    }

    pub async fn pause(&mut self) -> Result<(), VmError> {
        self.vm_state.ensure_is_running()?;

        {
            let mut vcpu_manager = self.vcpu_manager.lock().await;

            vcpu_manager.pause_all_vcpus().await?;
        }

        // TODO: pause devices

        self.vm_state = VmState::Paused;

        Ok(())
    }

    pub async fn resume(&mut self) -> Result<(), VmError> {
        self.vm_state.ensure_is_not_running()?;

        {
            let mut vcpu_manager = self.vcpu_manager.lock().await;

            vcpu_manager.resume_all_vcpus().await?;
        }

        // TODO: resume devices

        Ok(())
    }

    pub async fn save(&mut self, path: PathBuf) -> Result<(), VmSnapshotError> {
        self.vm_state.ensure_is_not_running()?;

        let mut tmp = NamedTempFile::new()?;

        let snap = self.build_snapshot().await?;

        let bytes = postcard::to_stdvec(&snap)?;
        tmp.write_all(&bytes)?;
        tmp.persist(&path).map_err(|e| e.error)?;

        Ok(())
    }
}
