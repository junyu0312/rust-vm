use std::sync::Arc;

use tokio::sync::Mutex;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::DTB_START;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_core::device_manager::DeviceManager;
use vm_core::virtualization::vm::HypervisorVm;
use vm_mm::manager::MemoryAddressSpace;

use crate::error::Result;
use crate::service::gdbstub::connection::VmGdbStubConnector;
use crate::service::monitor::MonitorServer;
use crate::vm::config::VmConfig;

pub mod config;

pub(crate) mod vm_exit_handler;

pub struct Vm {
    pub(crate) _vm_instance: Arc<dyn HypervisorVm>,
    pub(crate) vcpu_manager: Arc<Mutex<VcpuManager>>,
    pub(crate) memory_address_space: Arc<MemoryAddressSpace>,
    pub(crate) irq_chip: Arc<dyn InterruptController>,
    pub(crate) device_manager: Arc<DeviceManager>,
    pub(crate) gdb_stub: Option<VmGdbStubConnector>,
    pub(crate) monitor: MonitorServer,
    pub(crate) vm_config: VmConfig,
    pub(crate) start_pc: u64,
}

impl Vm {
    pub async fn boot(&mut self) -> Result<()> {
        self.monitor.start();

        if let Some(gdb_stub) = &self.gdb_stub {
            gdb_stub.wait_for_connection()?;
        }

        #[cfg(target_arch = "aarch64")]
        {
            self.vcpu_manager
                .lock()
                .await
                .boot_vcpu(0, self.start_pc, DTB_START)
                .await?;
        }

        #[cfg(not(target_arch = "aarch64"))]
        todo!();

        Ok(())
    }
}
