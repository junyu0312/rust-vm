use std::cell::OnceCell;

use tracing::warn;

pub struct VirtQueue {
    queue_size_max: u32,
    queue_size: u16,
    queue_ready: bool,
    queue_desc_low: OnceCell<u32>,
    queue_desc_high: OnceCell<u32>,
    queue_available_low: OnceCell<u32>,
    queue_available_high: OnceCell<u32>,
    queue_used_low: OnceCell<u32>,
    queue_used_high: OnceCell<u32>,
}

impl VirtQueue {
    pub fn new(queue_size_max: u32) -> Self {
        VirtQueue {
            queue_size_max,
            queue_size: Default::default(),
            queue_ready: Default::default(),
            queue_desc_low: Default::default(),
            queue_desc_high: Default::default(),
            queue_available_low: Default::default(),
            queue_available_high: Default::default(),
            queue_used_low: Default::default(),
            queue_used_high: Default::default(),
        }
    }

    pub fn read_queue_size_max(&self) -> u32 {
        self.queue_size_max
    }

    pub fn write_queue_size(&mut self, queue_size: u16) {
        self.queue_size = queue_size;
    }

    pub fn read_queue_ready(&self) -> bool {
        self.queue_ready
    }

    pub fn write_queue_ready(&mut self, queue_ready: bool) {
        self.queue_ready = queue_ready;
    }

    pub fn write_queue_desc_low(&mut self, addr: u32) {
        if self.queue_desc_low.set(addr).is_err() {
            warn!("repeated writes to queue_desc_low are ignored")
        }
    }

    pub fn write_queue_desc_high(&mut self, addr: u32) {
        if self.queue_desc_high.set(addr).is_err() {
            warn!("repeated writes to queue_desc_high are ignored")
        }
    }

    pub fn write_queue_available_low(&mut self, addr: u32) {
        if self.queue_available_low.set(addr).is_err() {
            warn!("repeated writes to queue_available_low are ignored")
        }
    }

    pub fn write_queue_available_high(&mut self, addr: u32) {
        if self.queue_available_high.set(addr).is_err() {
            warn!("repeated writes to queue_available_high are ignored")
        }
    }

    pub fn write_queue_used_low(&mut self, addr: u32) {
        if self.queue_used_low.set(addr).is_err() {
            warn!("repeated writes to queue_used_low are ignored")
        }
    }

    pub fn write_queue_used_high(&mut self, addr: u32) {
        if self.queue_used_high.set(addr).is_err() {
            warn!("repeated writes to queue_used_high are ignored")
        }
    }
}
