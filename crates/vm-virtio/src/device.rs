use crate::result::Result;
use crate::types::interrupt_status::InterruptStatus;
use crate::virt_queue::VirtQueue;

pub mod blk;

pub trait VirtIoDevice {
    const NAME: &str;
    const DEVICE_ID: u32;
    const VIRT_QUEUES_SIZE_MAX: &[u32];
    const DEVICE_FEATURES: u64;

    fn irq(&self) -> Option<u32>;

    fn trigger_irq(&self, active: bool);

    fn reset(&mut self);

    fn queue_notify(&mut self, virt_queues: &mut [VirtQueue], val: u32) -> Option<InterruptStatus>;

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()>;

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()>;
}
