use std::sync::Arc;
use std::sync::Mutex;

use vm_core::arch::irq::InterruptController;

use crate::device::virtqueue::VirtioConfigurationChangeNotifier;
use crate::device::virtqueue::VirtioUsedBufferNotifier;
use crate::types::interrupt_status::InterruptStatus;

pub struct VirtioMmioEventNotifier {
    irq_chip: Arc<dyn InterruptController>,
    irq: u32,
    is: Arc<Mutex<InterruptStatus>>,
    config_generation: Arc<Mutex<u8>>,
}

impl VirtioMmioEventNotifier {
    pub fn new(
        irq_chip: Arc<dyn InterruptController>,
        irq: u32,
        is: Arc<Mutex<InterruptStatus>>,
        config_generation: Arc<Mutex<u8>>,
    ) -> Self {
        VirtioMmioEventNotifier {
            irq_chip,
            irq,
            is,
            config_generation,
        }
    }
}

impl VirtioUsedBufferNotifier for VirtioMmioEventNotifier {
    fn notify_used_buffer(&self) {
        self.is
            .lock()
            .unwrap()
            .insert(InterruptStatus::VIRTIO_MMIO_INT_VRING);
        self.irq_chip.trigger_irq(self.irq, true);
    }
}

impl VirtioConfigurationChangeNotifier for VirtioMmioEventNotifier {
    fn update_config_generation(&self) {
        *self.config_generation.lock().unwrap() += 1;
        self.is
            .lock()
            .unwrap()
            .insert(InterruptStatus::VIRTIO_MMIO_INT_CONFIG);

        self.irq_chip.trigger_irq(self.irq, true);
    }
}
