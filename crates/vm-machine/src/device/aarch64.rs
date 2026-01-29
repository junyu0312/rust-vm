use std::sync::Arc;

use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_device::device::pl011::Pl011;

pub fn init_device(
    io_address_space: &mut IoAddressSpace,
    _irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()> {
    let pl011 = Pl011::new(MmioRange {
        start: 0x0900_0000,
        len: 0x1000,
    });

    io_address_space.register(Box::new(pl011))?;

    Ok(())
}
