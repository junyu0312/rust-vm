use std::sync::Arc;

use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_device::device::pl011::Pl011;
use vm_device::device::virtio::virtio_mmio_kbd::VirtIOMmioKbd;
use vm_device::virtio::VirtIoMmioAdaptor;

pub fn init_device(
    io_address_space: &mut IoAddressSpace,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()> {
    let pl011 = Pl011::new(MmioRange {
        start: 0x0900_0000,
        len: 0x1000,
    });

    let virtio_mmio_kbd = VirtIoMmioAdaptor::from(VirtIOMmioKbd::<48>::new(
        "virtio-mmio-kbd-01".to_string(),
        MmioRange {
            start: 0x0900_1000,
            len: 0x1000,
        },
        irq_chip,
    ));

    io_address_space.register(Box::new(pl011))?;
    io_address_space.register(Box::new(virtio_mmio_kbd))?;

    Ok(())
}
