use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::device_manager::DeviceManager;
use vm_core::irq::InterruptController;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_device::device::cmos::Cmos;
use vm_device::device::coprocessor::Coprocessor;
use vm_device::device::dummy::Dummy;
use vm_device::device::i8042::I8042;
use vm_device::device::pic::Pic;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart8250::Uart8250;
use vm_device::device::vga::Vga;
use vm_device::pci::root_complex::PciRootComplex;

pub fn init_device<C>(
    _mm: Arc<Mutex<MemoryAddressSpace<C>>>,
    device_manager: &mut DeviceManager,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()>
where
    C: MemoryContainer,
{
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

    let i8042 = I8042::new(irq_chip);

    device_manager.register_pio_device(Box::new(uart8250_com0))?;
    device_manager.register_pio_device(Box::new(uart8250_com1))?;
    device_manager.register_pio_device(Box::new(uart8250_com2))?;
    device_manager.register_pio_device(Box::new(uart8250_com3))?;
    device_manager.register_pio_device(Box::new(cmos))?;
    device_manager.register_pio_device(Box::new(post_debug))?;
    device_manager.register_pio_device(Box::new(coprocessor))?;
    device_manager.register_pio_device(Box::new(pic))?;
    device_manager.register_pio_device(Box::new(vga))?;
    device_manager.register_pio_device(Box::new(pci))?;
    device_manager.register_pio_device(Box::new(i8042))?;
    device_manager.register_pio_device(Box::new(Dummy))?;

    Ok(())
}
