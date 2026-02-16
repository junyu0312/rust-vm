/*
 * Interrupt flags (re: interrupt status & acknowledge registers)
 */

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Default)]
    pub struct InterruptStatus: u32 {
        const VIRTIO_MMIO_INT_VRING = 1 << 0;
        const VIRTIO_MMIO_INT_CONFIG = 1 << 1;
    }
}
