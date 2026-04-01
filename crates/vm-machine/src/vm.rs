use std::sync::Arc;

use vm_bootloader::boot_loader::BootLoader;
use vm_core::arch::irq::InterruptController;
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device_manager::manager::DeviceManager;
use vm_core::monitor::MonitorServer;
use vm_core::virt::Virt;
use vm_mm::manager::MemoryAddressSpace;

use crate::error::Error;
use crate::error::Result;

pub struct Vm<V: Virt> {
    pub(crate) ram_size: u64,
    pub(crate) vcpus: usize,
    pub(crate) memory_address_space: Arc<MemoryAddressSpace>,
    pub(crate) virt: V,
    pub(crate) irq_chip: Arc<dyn InterruptController>,
    pub(crate) device_manager: DeviceManager,
    pub(crate) gdb_stub: Option<GdbStub>,
    pub(crate) monitor: MonitorServer,
}

impl<V> Vm<V>
where
    V: Virt,
{
    pub fn run(&mut self, boot_loader: &dyn BootLoader) -> Result<()> {
        let start_pc = boot_loader.load(
            self.ram_size,
            self.vcpus,
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

        self.virt.run(start_pc, &self.device_manager)?;

        Ok(())
    }
}
