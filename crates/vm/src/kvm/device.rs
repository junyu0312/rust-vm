use anyhow::anyhow;
use vm_device::bus::io_address_space::IoAddressSpace;
use vm_device::device::cmos::Cmos;
use vm_device::device::coprocessor::Coprocessor;
use vm_device::device::pic::Pic;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart16550::Uart16550;
use vm_device::device::vga::Vga;
use vm_device::pci::host_bridge::PciHostBridge;

use crate::kvm::vm::KvmVm;

impl KvmVm {
    pub fn init_device(&mut self) -> anyhow::Result<()> {
        let uart16550 = Uart16550::default();

        let cmos = Cmos;

        let post_debug = PostDebug;

        let coprocessor = Coprocessor;

        let pic = Pic;

        let vga = Vga;

        let pci = PciHostBridge::default();

        let mut io_address_space = IoAddressSpace::default();
        io_address_space.register(Box::new(uart16550))?;
        io_address_space.register(Box::new(cmos))?;
        io_address_space.register(Box::new(post_debug))?;
        io_address_space.register(Box::new(coprocessor))?;
        io_address_space.register(Box::new(pic))?;
        io_address_space.register(Box::new(vga))?;
        io_address_space.register(Box::new(pci))?;

        self.io_address_space
            .set(io_address_space)
            .map_err(|_| anyhow!("pio_bus already set"))?;

        Ok(())
    }
}
