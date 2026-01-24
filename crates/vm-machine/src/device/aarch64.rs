use std::sync::Arc;

use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioLayout;
use vm_core::irq::InterruptController;
use vm_device::device::uart8250::Uart8250;

pub fn init_device(
    mmio_layout: MmioLayout,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<IoAddressSpace> {
    let uart8250 = Uart8250::<0x3f8, 33>::new(irq_chip);

    let mut io_address_space = IoAddressSpace::new(mmio_layout);
    io_address_space.register(Box::new(uart8250), Some((0x0900_0000, 0x1000)))?;

    Ok(io_address_space)
}
