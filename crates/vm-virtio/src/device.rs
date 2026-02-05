use crate::result::Result;

pub trait VirtIoDevice {
    const NAME: &str;
    const DEVICE_ID: u32;
    const VIRT_QUEUES_SIZE_MAX: &[u32];
    const DEVICE_FEATURES: u64;

    fn irq(&self) -> Option<u32>;

    fn reset(&mut self);

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()>;

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()>;
}
