use std::sync::Arc;
use std::sync::Mutex;

use vm_bootloader::boot_loader::BootLoader;
use vm_core::debug::gdbstub::GdbStub;
use vm_core::device::device_manager::DeviceManager;
use vm_core::virt::Virt;
use vm_mm::manager::MemoryAddressSpace;

use crate::error::Error;
use crate::error::Result;

pub struct Vm<V: Virt> {
    pub(crate) memory: Arc<Mutex<MemoryAddressSpace<V::Memory>>>,
    pub(crate) virt: V,
    pub(crate) device_manager: Arc<Mutex<DeviceManager>>,
    pub(crate) gdb_stub: Option<GdbStub>,
}

impl<V> Vm<V>
where
    V: Virt,
{
    pub fn run(&mut self, boot_loader: &dyn BootLoader<V>) -> Result<()> {
        {
            let mut memory = self.memory.lock().unwrap();

            let device_manager = self.device_manager.lock().unwrap();

            boot_loader.load(
                &mut self.virt,
                &mut memory,
                device_manager
                    .get_irq_chip()
                    .ok_or_else(|| Error::InitIrqchip("irq_chip is not exists".to_string()))?
                    .as_ref(),
                device_manager.mmio_devices(),
            )?;
        }

        if let Some(gdb_stub) = &self.gdb_stub {
            gdb_stub
                .wait_for_connection()
                .map_err(|err| Error::GdbStub(err.to_string()))?;
        }

        self.virt.run(self.device_manager.clone())?;

        Ok(())
    }
}
