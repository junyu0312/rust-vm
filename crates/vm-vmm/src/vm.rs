#[cfg(not(target_arch = "aarch64"))]
use std::hint::black_box;
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
    pub(crate) _memory_address_space: Arc<MemoryAddressSpace>,
    pub(crate) _irq_chip: Arc<dyn InterruptController>,
    pub(crate) _device_manager: Arc<DeviceManager>,
    pub(crate) gdb_stub: Option<VmGdbStubConnector>,
    pub(crate) monitor: MonitorServer,
    pub(crate) _vm_config: VmConfig,
    #[cfg(target_arch = "aarch64")]
    pub(crate) start_pc: u64,
}

impl Vm {
    pub async fn boot(&mut self) -> Result<()> {
        let mut stop_on_boot = false;

        self.monitor.start();

        if let Some(gdb_stub) = &self.gdb_stub {
            stop_on_boot = true;
            gdb_stub.wait_for_connection()?;
        }

        #[cfg(target_arch = "aarch64")]
        {
            let vcpu_manager = self.vcpu_manager.lock().await;

            let boot_vcpu = vcpu_manager.get_vcpu(0).unwrap();
            let mut boot_vcpu = boot_vcpu.lock().await;

            boot_vcpu
                .boot_vcpu(self.start_pc, DTB_START, stop_on_boot)
                .await?;
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            black_box(stop_on_boot);
            todo!();
        }

        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        {
            let vcpu_manager = self.vcpu_manager.lock().await;

            vcpu_manager.pause_all_vcpus().await?;
        }

        // TODO: pause devices

        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        {
            let vcpu_manager = self.vcpu_manager.lock().await;

            vcpu_manager.resume_all_vcpus().await?;
        }

        // TODO: resume devices

        Ok(())
    }
}
