use anyhow::anyhow;
use vm_device::bus::pio::PioBus;
use vm_device::device::cmos::Cmos;
use vm_device::device::coprocessor::Coprocessor;
use vm_device::device::pic::Pic;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart16550::Uart16550;
use vm_device::device::vga::Vga;

use crate::kvm::vm::KvmVm;

impl KvmVm {
    pub fn init_device(&mut self) -> anyhow::Result<()> {
        let uart16550 = Uart16550::default();

        let cmos = Cmos;

        let post_debug = PostDebug;

        let coprocessor = Coprocessor;

        let pic = Pic;

        let vga = Vga;

        let mut pio_bus = PioBus::default();
        pio_bus.register(Box::new(uart16550))?;
        pio_bus.register(Box::new(cmos))?;
        pio_bus.register(Box::new(post_debug))?;
        pio_bus.register(Box::new(coprocessor))?;
        pio_bus.register(Box::new(pic))?;
        pio_bus.register(Box::new(vga))?;

        self.pio_bus
            .set(pio_bus)
            .map_err(|_| anyhow!("pio_bus already set"))?;

        Ok(())
    }
}
