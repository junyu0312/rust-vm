use std::sync::Arc;

use vm_bootloader::boot_loader::BootLoader;
use vm_core::arch::irq::InterruptController;
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device::device_manager::DeviceManager;
use vm_core::monitor::MonitorServer;
use vm_core::virt::Virt;
use vm_mm::manager::MemoryAddressSpace;

use crate::error::Error;
use crate::error::Result;

pub struct Vm<V: Virt> {
    pub(crate) memory: Arc<MemoryAddressSpace<V::Memory>>,
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
    pub fn run(&mut self, boot_loader: &dyn BootLoader<V>) -> Result<()> {
        boot_loader.load(
            &mut self.virt,
            &self.memory,
            self.irq_chip.as_ref(),
            self.device_manager.mmio_devices(),
        )?;

        self.monitor.start();

        if let Some(gdb_stub) = &self.gdb_stub {
            gdb_stub
                .wait_for_connection()
                .map_err(|err| Error::GdbStub(err.to_string()))?;
        }

        self.virt.run(&self.device_manager)?;

        Ok(())
    }
}
