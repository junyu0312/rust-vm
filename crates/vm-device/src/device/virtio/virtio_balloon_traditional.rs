use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_mm::allocator::MemoryContainer;
use vm_mm::manager::MemoryAddressSpace;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::transport::TransportContext;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::device::virtqueue::VirtqueueHandlerFn;
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

fn inflateq_handler<C>() -> VirtqueueHandlerFn<C, VirtioBalloonTranditional<C>>
where
    C: MemoryContainer,
{
    Box::new(|mm, dev, desc_ring, desc_id| {
        let desc = desc_ring.get(desc_id);
        let len = desc.len;
        assert!(len.is_multiple_of(4));

        let array = desc.addr(mm).unwrap().as_ptr() as *const u32;

        for i in 0..(len / 4) {
            let pfn = unsafe { *array.add(i as usize) };
            assert!(dev.device.balloon.insert(pfn));
            let gpa = (pfn as u64) << 12;
            let _hva = mm.gpa_to_hva(gpa).unwrap();
            // TODO: mmap
        }

        len
    })
}

fn deflateq_handler<C>() -> VirtqueueHandlerFn<C, VirtioBalloonTranditional<C>>
where
    C: MemoryContainer,
{
    Box::new(|mm, dev, desc_ring, desc_id| {
        let desc = desc_ring.get(desc_id);
        let len = desc.len;
        assert!(len.is_multiple_of(4));

        let array = desc.addr(mm).unwrap().as_ptr() as *const u32;

        for i in 0..(len / 4) {
            let pfn = unsafe { *array.add(i as usize) };
            assert!(dev.device.balloon.remove(&pfn));
            let gpa = (pfn as u64) << 12;
            let _hva = mm.gpa_to_hva(gpa).unwrap();
            // TODO: mmap
        }

        len
    })
}

pub struct VirtioBalloonTranditional<C>
where
    C: MemoryContainer,
{
    irq: u32,
    irq_chip: Arc<dyn InterruptController>,
    mm: Arc<MemoryAddressSpace<C>>,
    cfg: VirtioBalloonTranditionalConfig,
    balloon: HashSet<u32>,
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
            balloon: Default::default(),
        }
    }

    pub fn set_num_pages(&mut self, num_pages: u32) {
        self.cfg.num_pages = num_pages;

        todo!("notify config generation changes");
    }
}

impl<C> VirtioDevice<C> for VirtioBalloonTranditional<C>
where
    C: MemoryContainer,
{
    const NAME: &str = "virtio-balloon-tranditional";
    const DEVICE_ID: u16 = DeviceId::Balloon as u16;
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
        dev: Arc<Mutex<VirtioDev<C, Self>>>,
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

    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<()> {
        self.cfg.as_mut_bytes()[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    fn transport_context(&self) -> &dyn TransportContext {
        todo!()
    }

    fn transport_context_mit(&mut self) -> &mut dyn TransportContext {
        todo!()
    }
}

pub trait VirtioBalloonApi {
    fn update_num_pages(&mut self, num_pages: u32);
}

pub type VirtioBalloonDev<C> = VirtioDev<C, VirtioBalloonTranditional<C>>;

impl<C> VirtioBalloonApi for VirtioBalloonDev<C>
where
    C: MemoryContainer,
{
    fn update_num_pages(&mut self, num_pages: u32) {
        if self.device.cfg.num_pages == num_pages {
            return;
        }

        self.device.cfg.num_pages = num_pages;
        self.update_config_generation_and_notify();
    }
}

pub type VirtioMmioBalloonDevice<C> = VirtioMmioTransport<C, VirtioBalloonTranditional<C>>;
