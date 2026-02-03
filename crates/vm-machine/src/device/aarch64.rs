use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_device::device::pl011::Pl011;

pub fn init_device<C>(
    _mm: Arc<Mutex<MemoryAddressSpace<C>>>,
    io_address_space: &mut IoAddressSpace,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()>
where
    C: MemoryContainer,
{
    let pl011 = Pl011::<1>::new(
        MmioRange {
            start: 0x0900_0000,
            len: 0x1000,
        },
        irq_chip,
    );

    // let virtio_mmio_kbd = VirtIOMmioKbd::<48, C>::new(
    //     mm,
    //     "virtio-mmio-kbd-01".to_string(),
    //     MmioRange {
    //         start: 0x0900_1000,
    //         len: 0x1000,
    //     },
    //     irq_chip,
    //     rx,
    // );

    io_address_space.register(Box::new(pl011))?;
    // io_address_space.register(Box::new(virtio_mmio_kbd))?;

    Ok(())
}
