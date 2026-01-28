use std::sync::Arc;

use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_device::device::uart8250::Uart8250;

pub fn init_device(
    io_address_space: &mut IoAddressSpace,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()> {
    let uart8250 = Uart8250::<33>::new(
        None,
        Some(MmioRange {
            start: 0x09000000,
            len: 0x1000,
        }),
        irq_chip,
    );

    io_address_space.register(Box::new(uart8250))?;

    Ok(())
}
