use std::slice;
use std::sync::Arc;
use std::sync::Mutex;

use rand::Rng;
use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::device::virtqueue::VirtqueueHandlerFn;
use vm_virtio::result::Result;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::mmio::VirtioMmioTransport;
use vm_virtio::transport::pci::VirtioPciDevice;
use vm_virtio::types::device::entropy::VirtioEntropyConfig;
use vm_virtio::types::device::entropy::VirtioEntropyVirtqueue;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use vm_virtio::virtqueue::virtq_desc_table::VIRTQ_DESC_F_WRITE;

fn requestq_handler<C>() -> VirtqueueHandlerFn<C, VirtioEntropy<C>>
where
    C: MemoryContainer,
{
    Box::new(|mm, _dev, desc_ring, desc_id| {
        let desc = desc_ring.get(desc_id);
        let len = desc.len;

        let mut rng = rand::rng();

        let buf = desc.addr(mm).unwrap().as_ptr();

        // println!("entropy len: {} {desc_id}", len);
        assert!(desc.flags & VIRTQ_DESC_F_WRITE != 0);

        unsafe {
            rng.fill_bytes(slice::from_raw_parts_mut(buf, len as usize));
        }

        len
    })
}

pub struct VirtioEntropy<C>
where
    C: MemoryContainer,
{
    irq: u32,
    irq_chip: Arc<dyn InterruptController>,
    mm: Arc<MemoryAddressSpace<C>>,
}

impl<C> VirtioEntropy<C>
where
    C: MemoryContainer,
{
    pub fn new(
        irq: u32,
        irq_chip: Arc<dyn InterruptController>,
        mm: Arc<MemoryAddressSpace<C>>,
    ) -> Self {
        VirtioEntropy { irq, irq_chip, mm }
    }
}

impl<C> VirtioDevice<C> for VirtioEntropy<C>
where
    C: MemoryContainer,
{
    const NAME: &str = "virtio-entropy";
    const DEVICE_ID: u16 = DeviceId::Entropy as u16;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<Option<u32>> {
        vec![Some(8)]
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
        if queue != 0 {
            return None;
        }

        Some(VirtqueueHandler {
            queue_sel: VirtioEntropyVirtqueue::Requestq as usize,
            notifier,
            dev,
            mm: self.mm.clone(),
            irq_chip: self.irq_chip.clone(),
            irq_line: self.irq + 32,
            handle_desc: requestq_handler(),
        })
    }

    fn read_config(&self, _offset: usize, _buf: &mut [u8]) -> Result<()> {
        Ok(()) // no cfg for entropy device
    }

    fn write_config(&mut self, _offset: usize, _buf: &[u8]) -> Result<()> {
        Ok(()) // no cfg for entropy device
    }
}

impl<C> VirtioPciDevice<C> for VirtioEntropy<C>
where
    C: MemoryContainer,
{
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioEntropyConfig>();
    const CLASS_CODE: u32 = 0x000000;
    const IRQ_PIN: u8 = InterruptPin::INTA as u8;
}

pub type VirtioMmioEntropyDevice<C> = VirtioMmioTransport<C, VirtioEntropy<C>>;
