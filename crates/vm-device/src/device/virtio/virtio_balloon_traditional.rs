use std::sync::Arc;

use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_mm::allocator::MemoryContainer;
use vm_mm::manager::MemoryAddressSpace;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::VirtqueueHandler;
use vm_virtio::device::VirtqueueHandlerFn;
use vm_virtio::result::Result;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::mmio::VirtioMmioTransport;
use vm_virtio::types::device::balloon_tranditional::VirtioBalloonTranditionalConfig;
use vm_virtio::types::device::balloon_tranditional::VirtioBalloonTranditionalVirtqueue;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use zerocopy::IntoBytes;

const INFLATEQ_QUEUE_SIZE_MAX: u32 = 512;
const DEFLATEQ_QUEUE_SIZE_MAX: u32 = 512;

fn inflateq_handler<C, D>() -> VirtqueueHandlerFn<C, D> {
    Box::new(|_mm, _dev, _desc_ring, _desc_id| todo!())
}

fn deflateq_handler<C, D>() -> VirtqueueHandlerFn<C, D> {
    Box::new(|_mm, _dev, _desc_ring, _desc_id| todo!())
}

pub struct VirtioBalloonTranditional<C>
where
    C: MemoryContainer,
{
    irq: u32,
    irq_chip: Arc<dyn InterruptController>,
    mm: Arc<MemoryAddressSpace<C>>,
    cfg: VirtioBalloonTranditionalConfig,
}

impl<C> VirtioBalloonTranditional<C>
where
    C: MemoryContainer,
{
    pub fn new(
        irq: u32,
        irq_chip: Arc<dyn InterruptController>,
        mm: Arc<MemoryAddressSpace<C>>,
    ) -> Self {
        VirtioBalloonTranditional {
            irq,
            irq_chip,
            mm,
            cfg: VirtioBalloonTranditionalConfig::default(),
        }
    }
}

impl<C> VirtioDevice<C> for VirtioBalloonTranditional<C>
where
    C: MemoryContainer,
{
    const NAME: &str = "virtio-balloon-tranditional";
    const DEVICE_ID: u32 = DeviceId::Balloon as u32;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<Option<u32>> {
        vec![Some(INFLATEQ_QUEUE_SIZE_MAX), Some(DEFLATEQ_QUEUE_SIZE_MAX)]
    }

    fn irq(&self) -> Option<u32> {
        Some(self.irq)
    }

    fn trigger_irq(&self, active: bool) {
        self.irq_chip.trigger_irq(32 + self.irq, active);
    }

    fn reset(&mut self) {}

    fn virtqueue_handler(
        &self,
        queue: usize,
        notifier: Arc<Notify>,
        dev: VirtioDev<C, Self>,
    ) -> Option<VirtqueueHandler<C, Self>> {
        match VirtioBalloonTranditionalVirtqueue::from_repr(queue) {
            Some(virtq) => match virtq {
                VirtioBalloonTranditionalVirtqueue::Inflateq => Some(VirtqueueHandler {
                    queue_sel: VirtioBalloonTranditionalVirtqueue::Inflateq as usize,
                    notifier,
                    dev,
                    mm: self.mm.clone(),
                    irq_chip: self.irq_chip.clone(),
                    irq_line: self.irq + 32,
                    handle_desc: inflateq_handler(),
                }),
                VirtioBalloonTranditionalVirtqueue::Defalteq => Some(VirtqueueHandler {
                    queue_sel: VirtioBalloonTranditionalVirtqueue::Defalteq as usize,
                    notifier,
                    dev,
                    mm: self.mm.clone(),
                    irq_chip: self.irq_chip.clone(),
                    irq_line: self.irq + 32,
                    handle_desc: deflateq_handler(),
                }),
                VirtioBalloonTranditionalVirtqueue::Statsq => None,
                VirtioBalloonTranditionalVirtqueue::FreePageVq => None,
                VirtioBalloonTranditionalVirtqueue::ReportingVq => None,
            },
            None => None,
        }
    }

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + len]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()> {
        self.cfg.as_mut_bytes()[offset..len].copy_from_slice(buf);
        Ok(())
    }
}

pub type VirtioMmioBalloonDevice<C> = VirtioMmioTransport<C, VirtioBalloonTranditional<C>>;
