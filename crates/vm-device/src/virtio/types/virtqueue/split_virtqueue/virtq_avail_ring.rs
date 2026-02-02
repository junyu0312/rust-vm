pub const VIRTQ_AVAIL_F_NO_INTERRUPT: u16 = 1;

// It is only written by the driver and read by the device.
pub struct VirtqAvail {
    queue_size: u16,
    buf: *const u16,
}

impl VirtqAvail {
    pub fn new(queue_size: u16, buf: *const u16) -> Self {
        VirtqAvail { queue_size, buf }
    }

    pub fn flags(&self) -> u16 {
        unsafe { *(self.buf) }
    }

    pub fn idx(&self) -> u16 {
        unsafe { *((self.buf).add(1)) }
    }

    pub fn ring(&self, idx: u16) -> u16 {
        unsafe { *((self.buf).add(2 + idx as usize)) }
    }

    /// Only if VIRTIO_F_EVENT_IDX
    pub fn used_event(&self) -> u16 {
        unsafe { *((self.buf).add(2 + self.queue_size as usize)) }
    }
}
