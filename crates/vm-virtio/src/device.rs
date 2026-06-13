use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_core::device::error::DeviceSnapshotError;

use crate::device::virtqueue::VirtqueueHandler;
use crate::result::VirtioError;
use crate::transport::VirtioDev;

pub mod transport;
pub mod virtqueue;

pub trait VirtioDevice: Sized + Send + Sync + 'static {
    const NAME: &str;
    const DEVICE_ID: u16;
    const DEVICE_FEATURES: u64;

    fn virtqueues_size_max(&self) -> Vec<Option<u32>>;

    fn num_queues(&self) -> u16 {
        self.virtqueues_size_max()
            .iter()
            .filter(|v| v.is_some())
            .count()
            .try_into()
            .unwrap()
    }

    fn irq(&self) -> u32;

    fn irq_chip(&self) -> &dyn InterruptController;

    fn reset(&mut self);

    fn virtqueue_handler(
        &self,
        queue: usize,
        notifier: Arc<Notify>,
        dev: Arc<Mutex<VirtioDev<Self>>>,
    ) -> Option<VirtqueueHandler<Self>>;

    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<(), VirtioError>;

    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<(), VirtioError>;

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }
}
