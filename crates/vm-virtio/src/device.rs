use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_mm::allocator::MemoryContainer;
use vm_mm::manager::MemoryAddressSpace;

use crate::result::Result;
use crate::transport::VirtioDev;
use crate::types::interrupt_status::InterruptStatus;
use crate::virtqueue::virtq_desc_table::VirtqDescTableRef;

pub mod blk;
pub mod pci;

pub type VirtqueueHandlerFn<C, D> = Box<
    dyn Fn(&MemoryAddressSpace<C>, &mut VirtioDev<C, D>, &VirtqDescTableRef, u16) -> u32
        + Send
        + Sync,
>;

pub struct VirtqueueHandler<C, D> {
    pub queue_sel: usize,
    pub notifier: Arc<Notify>,
    pub dev: Arc<Mutex<VirtioDev<C, D>>>,
    pub mm: Arc<MemoryAddressSpace<C>>,
    pub irq_chip: Arc<dyn InterruptController>,
    pub irq_line: u32,
    pub handle_desc: VirtqueueHandlerFn<C, D>,
}

impl<C, D> VirtqueueHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioDevice<C>,
{
    pub async fn run(self) {
        let mm = self.mm.as_ref();

        loop {
            self.notifier.notified().await;

            let mut dev = self.dev.lock().unwrap();

            let mut updated = false;
            loop {
                let (desc_table, desc_id) = {
                    // fetch desc from avail ring
                    let q = dev.get_virtqueue_mut(self.queue_sel).unwrap();
                    let avail_ring = q.avail_ring(mm).unwrap();

                    if q.last_available_idx() == avail_ring.idx() {
                        break;
                    }

                    let last_available_idx = q.last_available_idx();
                    let desc_id = avail_ring.ring(last_available_idx);
                    q.incr_last_available_idx();
                    (q.desc_table_ref(mm).unwrap(), desc_id)
                };

                let len = (self.handle_desc)(mm, &mut dev, &desc_table, desc_id);

                {
                    // update used ring
                    let q = dev.get_virtqueue_mut(self.queue_sel).unwrap();

                    let mut used_ring = q.used_ring(mm).unwrap();
                    let used_idx = used_ring.idx();
                    let used_entry = used_ring.ring(used_idx);
                    used_entry.id = desc_id as u32;
                    used_entry.len = len;
                    used_ring.incr_idx();
                }

                updated = true;
            }

            if updated {
                // update irq
                let mut isr = dev.get_interrupt_status();
                isr.insert(InterruptStatus::VIRTIO_MMIO_INT_VRING);
                dev.set_interrupt_status(isr);
                self.irq_chip.trigger_irq(self.irq_line, true);
            }
        }
    }
}

pub trait VirtioDevice<C>: Sized + Send + Sync + 'static {
    const NAME: &str;
    const DEVICE_ID: u32;
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

    fn irq(&self) -> Option<u32>;

    fn trigger_irq(&self, active: bool);

    fn reset(&mut self);

    fn virtqueue_handler(
        &self,
        queue: usize,
        notifier: Arc<Notify>,
        dev: Arc<Mutex<VirtioDev<C, Self>>>,
    ) -> Option<VirtqueueHandler<C, Self>>;

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()>;

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()>;
}
