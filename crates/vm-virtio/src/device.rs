use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use tokio::runtime::Handle;
use vm_core::arch::irq::InterruptController;
use vm_core::device::error::DeviceSnapshotError;
use vm_core::virtualization::irq_allocator::IrqAllocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;

use crate::device::virtqueue::VirtqueueHandler;
use crate::result::Result;
use crate::result::VirtioError;
use crate::transport::common::VirtioTransportCommon;
use crate::transport::mmio::VirtioMmioTransport;

pub mod virtqueue;

/// Define the device-specific behavior of a virtio device
pub trait VirtioDevice: Sized + Send + Sync + 'static {
    const NAME: &str;
    const DEVICE_ID: u16;
    const DEVICE_FEATURES: u64;

    fn virtqueues_size_max(&self) -> Vec<u16>;

    /// A virtio device can have maximum of 65536 virtqueues.
    fn num_queues(&self) -> u16 {
        self.virtqueues_size_max()
            .len()
            .try_into()
            .unwrap_or_else(|_| panic!("virtio device {} has too many virtqueues", Self::NAME))
    }

    fn reset(&mut self);

    fn virtqueue_handler(&self, queue_sel: u16) -> Option<Box<dyn VirtqueueHandler>>;

    /// Read to device-specific configuration
    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<()>;

    /// Write to device-specific configuration
    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<()>;

    fn pause(&self) -> std::result::Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn resume(&self) -> std::result::Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn save(&self, _writer: &mut dyn Write) -> std::result::Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn load(&mut self, _reader: &mut dyn Read) -> std::result::Result<(), DeviceSnapshotError> {
        Err(DeviceSnapshotError::DeviceNotSupportSnapshot(
            Self::NAME.to_string(),
        ))
    }

    fn into_mmio_device(
        self,
        mmio_allocator: &mut RangeAllocator<u64>,
        irq_allocator: &mut IrqAllocator,
        virtio_aml_path_allocator: &mut RangeAllocator<u8>,
        tokio_runtime: Handle,
        memory: Arc<MemoryAddressSpace>,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<VirtioMmioTransport<Self>> {
        let mmio_range = mmio_allocator
            .alloc(0x1000)
            .map_err(VirtioError::AllocMmioRange)?;

        let id = virtio_aml_path_allocator
            .alloc(1)
            .map_err(VirtioError::AllocIrq)?;

        let dev = VirtioMmioTransport::new(
            tokio_runtime,
            memory,
            irq_chip,
            id.start,
            mmio_range,
            irq_allocator.alloc().unwrap().try_into().unwrap(),
            VirtioTransportCommon::new(self)?,
        );

        Ok(dev)
    }
}
