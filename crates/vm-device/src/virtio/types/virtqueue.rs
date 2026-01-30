#[derive(Default)]
pub struct VirtQueue<const QUEUE_SIZE_MAX: u32> {
    queue_size: u32,
    queue_ready: bool,
    queue_desc_low: u32,
    queue_desc_high: u32,
    queue_avail_low: u32,
    queue_avail_high: u32,
    queue_used_low: u32,
    queue_used_high: u32,
}

impl<const QUEUE_SIZE_MAX: u32> VirtQueue<QUEUE_SIZE_MAX> {
    pub fn queue_size_max(&self) -> u32 {
        QUEUE_SIZE_MAX
    }

    pub fn write_queue_size(&mut self, size: u32) {
        self.queue_size = size;
    }

    pub fn read_queue_ready(&self) -> bool {
        self.queue_ready
    }

    pub fn write_queue_ready(&mut self, ready: bool) {
        self.queue_ready = ready
    }

    pub fn write_queue_desc_low(&mut self, addr: u32) {
        self.queue_desc_low = addr
    }

    pub fn write_queue_desc_high(&mut self, addr: u32) {
        self.queue_desc_high = addr
    }

    pub fn write_queue_avail_low(&mut self, addr: u32) {
        self.queue_avail_low = addr
    }

    pub fn write_queue_avail_high(&mut self, addr: u32) {
        self.queue_avail_high = addr
    }

    pub fn write_queue_used_low(&mut self, addr: u32) {
        self.queue_used_low = addr
    }

    pub fn write_queue_used_high(&mut self, addr: u32) {
        self.queue_used_high = addr
    }
}
