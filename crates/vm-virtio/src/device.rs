use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::Notify;

use crate::result::Result;
use crate::types::interrupt_status::InterruptStatus;
use crate::virt_queue::VirtQueue;

pub mod blk;
pub mod pci;

#[async_trait]
pub trait VirtQueueHandler {
    async fn handler(&self, virt_queue: &mut VirtQueue);
}

pub trait VirtIoDevice: Sized + 'static {
    const NAME: &str;
    const DEVICE_ID: u32;
    const VIRT_QUEUES_SIZE_MAX: &[u32];
    const DEVICE_FEATURES: u64;

    fn irq(&self) -> Option<u32>;

    fn trigger_irq(&self, active: bool);

    fn reset(&mut self);

    fn virtqueue_handler(
        &self,
        queue: usize,
        notify: Arc<Notify>,
        interrupt_status: Arc<Mutex<InterruptStatus>>,
        virtqueue: Arc<Mutex<VirtQueue>>,
    ) -> Option<impl Future<Output = ()> + Send + 'static>;

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()>;

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()>;
}
