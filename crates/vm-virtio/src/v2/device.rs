pub enum DeviceId {
    Blk = 2,
}

pub trait VirtIoDevice {
    const NAME: &str;
    const DEVICE_ID: u32;
    const VIRT_QUEUES_SIZE_MAX: &[u32];
    const DEVICE_FEATURES: u64;

    fn irq(&self) -> Option<u32>;

    fn reset(&mut self);
}
