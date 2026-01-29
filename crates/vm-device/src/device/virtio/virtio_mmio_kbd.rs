use std::sync::Arc;

use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;

use crate::virtio::device::virtio_input::VirtIOInput;
use crate::virtio::transport::mmio::VirtIoMmio;

pub struct VirtIOMmioKbd<const IRQ: u32> {
    mmio_range: MmioRange,
    _irq_chip: Arc<dyn InterruptController>,
}

impl<const IRQ: u32> VirtIOMmioKbd<IRQ> {
    pub fn new(mmio_range: MmioRange, irq_chip: Arc<dyn InterruptController>) -> Self {
        VirtIOMmioKbd {
            mmio_range,
            _irq_chip: irq_chip,
        }
    }
}

impl<const IRQ: u32> VirtIOInput for VirtIOMmioKbd<IRQ> {}

impl<const IRQ: u32> VirtIoMmio for VirtIOMmioKbd<IRQ> {
    type Subsystem = Self;

    const NAME: &str = "virtio-mmio-kbd";

    fn mmio_range(&self) -> &MmioRange {
        &self.mmio_range
    }

    fn interrupts(&self) -> Option<&[u32]> {
        Some(&[0, IRQ, 4])
    }
}
