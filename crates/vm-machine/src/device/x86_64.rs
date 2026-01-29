use std::sync::Arc;
use std::sync::mpsc;

use vm_core::device::IoAddressSpace;
use vm_core::irq::InterruptController;
use vm_device::device::cmos::Cmos;
use vm_device::device::coprocessor::Coprocessor;
use vm_device::device::dummy::Dummy;
use vm_device::device::i8042::I8042;
use vm_device::device::pic::Pic;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart8250::Uart8250;
use vm_device::device::vga::Vga;
use vm_device::pci::root_complex::PciRootComplex;

use crate::utils::stdin::init_stdin;

pub fn init_device(
    io_address_space: &mut IoAddressSpace,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()> {
    let uart8250_com0 = Uart8250::<4>::new(Some(0x3f8), None, irq_chip.clone());
    let uart8250_com1 = Uart8250::<3>::new(Some(0x2f8), None, irq_chip.clone());
    let uart8250_com2 = Uart8250::<4>::new(Some(0x3e8), None, irq_chip.clone());
    let uart8250_com3 = Uart8250::<3>::new(Some(0x2e8), None, irq_chip.clone());

    let cmos = Cmos;

    let post_debug = PostDebug;

    let coprocessor = Coprocessor;

    let pic = Pic;

    let vga = Vga;

    let pci = PciRootComplex::default();

    let (tx, rx) = mpsc::channel();
    init_stdin(tx)?;
    let i8042 = I8042::new(irq_chip, rx);

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
    io_address_space.register(Box::new(i8042))?;
    io_address_space.register(Box::new(Dummy))?;

    Ok(())
}
