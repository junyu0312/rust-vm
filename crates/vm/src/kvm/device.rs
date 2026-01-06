use std::sync::Arc;

use anyhow::anyhow;
use vm_device::bus::io_address_space::IoAddressSpace;
use vm_device::device::cmos::Cmos;
use vm_device::device::coprocessor::Coprocessor;
use vm_device::device::dummy::Dummy;
use vm_device::device::i8042::I8042;
use vm_device::device::pic::Pic;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart8250::Uart8250;
use vm_device::device::vga::Vga;
use vm_device::pci::host_bridge::PciHostBridge;

use crate::kvm::irq::KvmIRQ;
use crate::kvm::vm::KvmVm;

impl KvmVm {
    pub fn init_device(&mut self, irq_chip: Arc<KvmIRQ>) -> anyhow::Result<()> {
        let uart8250_com0 = Uart8250::<0x3f8, 4>::new(irq_chip.clone());
        let uart8250_com1 = Uart8250::<0x2f8, 3>::new(irq_chip.clone());
        let uart8250_com2 = Uart8250::<0x3e8, 4>::new(irq_chip.clone());
        let uart8250_com3 = Uart8250::<0x2e8, 3>::new(irq_chip);

        let cmos = Cmos;

        let post_debug = PostDebug;

        let coprocessor = Coprocessor;

        let pic = Pic;

        let vga = Vga;

        let pci = PciHostBridge::default();

        let mut io_address_space = IoAddressSpace::default();
        io_address_space.register(Box::new(uart8250_com0))?;
        io_address_space.register(Box::new(uart8250_com1))?;
        io_address_space.register(Box::new(uart8250_com2))?;
        io_address_space.register(Box::new(uart8250_com3))?;
        io_address_space.register(Box::new(cmos))?;
        io_address_space.register(Box::new(post_debug))?;
        io_address_space.register(Box::new(coprocessor))?;
        io_address_space.register(Box::new(pic))?;
        io_address_space.register(Box::new(vga))?;
        io_address_space.register(Box::new(pci))?;

        #[cfg(target_arch = "x86_64")]
        {
            let i8042 = I8042::default();
            io_address_space.register(Box::new(i8042))?;
        }

        io_address_space.register(Box::new(Dummy))?;

        self.io_address_space
            .set(io_address_space)
            .map_err(|_| anyhow!("pio_bus already set"))?;

        Ok(())
    }
}
