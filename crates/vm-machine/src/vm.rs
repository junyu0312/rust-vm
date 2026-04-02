use std::sync::Arc;
use std::sync::Mutex;

use vm_bootloader::boot_loader::BootLoader;
#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::DTB_START;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device_manager::DeviceManager;
use vm_core::virt::vm::HypervisorVm;
use vm_mm::manager::MemoryAddressSpace;

use crate::error::Error;
use crate::error::Result;
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
    pub(crate) gdb_stub: Option<GdbStub>,
    pub(crate) monitor: MonitorServer,
    pub(crate) vm_config: VmConfig,
}

impl Vm {
    pub fn boot(&mut self, boot_loader: &dyn BootLoader) -> Result<()> {
        let start_pc = boot_loader.load(
            self.vm_config.memory_size as u64,
            self.vm_config.vcpus,
            &self.memory_address_space,
            self.irq_chip.as_ref(),
            self.device_manager.mmio_devices(),
        )?;

        self.monitor.start();

        if let Some(gdb_stub) = &self.gdb_stub {
            gdb_stub
                .wait_for_connection()
                .map_err(|err| Error::GdbStub(err.to_string()))?;
        }

        #[cfg(target_arch = "aarch64")]
        self.vcpu_manager
            .lock()
            .unwrap()
            .start_vcpu(0, start_pc, DTB_START)?;

        #[cfg(not(target_arch = "aarch64"))]
        todo!();

        Ok(())
    }
}
