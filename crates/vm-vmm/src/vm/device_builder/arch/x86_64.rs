use vm_core::arch::x86_64::layout::*;
use vm_device::device::cmos::Cmos;
use vm_device::device::dummy::Dummy;
use vm_device::device::post_debug::PostDebug;
use vm_device::device::uart8250::Uart8250;
use vm_utils::range_allocator::RangeAllocator;

use crate::device::error::InitDeviceError;
use crate::vm::device_builder::DeviceManagerBuilder;

pub fn pio_allocator() -> RangeAllocator<u16> {
    let mut allocator = RangeAllocator::<u16>::default();

    allocator.insert(IO_PORT_START, IO_PORT_LEN).unwrap();

    allocator
}

pub fn mmio_allocator() -> RangeAllocator<u64> {
    let mut allocator = RangeAllocator::<u64>::default();

    allocator
        .insert(MMIO_START as u64, MMIO_LEN as usize)
        .unwrap();
    allocator
        .insert(
            PCI_BAR_MMIO_WINDOW_START as u64,
            PCI_BAR_MMIO_WINDOW_LENGTH as usize,
        )
        .unwrap();
    allocator
        .insert(ECAM_BASE as u64, ECAM_LENGTH as usize)
        .unwrap();

    allocator
}

impl<'a> DeviceManagerBuilder<'a> {
    pub fn init_device_arch(&mut self) -> Result<(), InitDeviceError> {
        let uart8250_com1 =
            Uart8250::<4>::new(&mut self.pio_allocator, 0x3f8, self.irq_chip.clone(), true)?;
        self.device_manager.attach_device(Box::new(uart8250_com1))?;

        let uart8250_com2 =
            Uart8250::<3>::new(&mut self.pio_allocator, 0x2f8, self.irq_chip.clone(), false)?;
        self.device_manager.attach_device(Box::new(uart8250_com2))?;

        let uart8250_com3 =
            Uart8250::<4>::new(&mut self.pio_allocator, 0x3e8, self.irq_chip.clone(), false)?;
        self.device_manager.attach_device(Box::new(uart8250_com3))?;

        let uart8250_com4 =
            Uart8250::<3>::new(&mut self.pio_allocator, 0x2e8, self.irq_chip.clone(), false)?;
        self.device_manager.attach_device(Box::new(uart8250_com4))?;

        let cmos = Cmos::new(&mut self.pio_allocator)?;
        self.device_manager.attach_device(Box::new(cmos))?;

        let post_debug = PostDebug::new(&mut self.pio_allocator)?;
        self.device_manager.attach_device(Box::new(post_debug))?;

        let dummy = Dummy::new(&mut self.pio_allocator)?;
        self.device_manager.attach_device(Box::new(dummy))?;

        // let i8042 = I8042::new(self.irq_chip.clone());
        // self.device_manager.attach_device(Box::new(i8042))?;

        Ok(())
    }
}
