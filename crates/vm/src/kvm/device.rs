use anyhow::anyhow;
use vm_device::bus::pio::PioBus;
use vm_device::device::uart16550::Uart16550;

use crate::kvm::vm::KvmVm;

impl KvmVm {
    pub fn init_device(&mut self) -> anyhow::Result<()> {
        let uart16550 = Uart16550::default();

        let mut pio_bus = PioBus::default();
        pio_bus.register(Box::new(uart16550))?;

        self.pio_bus
            .set(pio_bus)
            .map_err(|_| anyhow!("pio_bus already set"))?;

        Ok(())
    }
}
