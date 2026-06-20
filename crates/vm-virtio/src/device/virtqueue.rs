use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::fence;

use async_trait::async_trait;
use tokio::select;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use vm_mm::manager::MemoryAddressSpace;

use crate::virtqueue::Virtqueue;
use crate::virtqueue::virtq_desc_table::VirtqDescTableRef;

#[async_trait]
pub trait VirtqueueHandler: Send + Sync {
    async fn handle_desc(&self, desc_ring: &VirtqDescTableRef, desc_id: u16) -> u32;
}

pub trait VirtioUsedBufferNotifier: Send + Sync {
    fn notify_used_buffer(&self);
}

pub trait VirtioConfigurationChangeNotifier: Send + Sync {
    fn update_config_generation(&self);
}

#[derive(Default)]
pub struct VirtqueueWorkerController {
    pub queue_notify: Arc<Notify>,
    pub queue_disable: Arc<CancellationToken>,
}

pub async fn virtqueue_worker(
    mm: Arc<MemoryAddressSpace>,
    controller: Arc<VirtqueueWorkerController>,
    used_buffer_notification: Arc<dyn VirtioUsedBufferNotifier>,
    virtqueue: Virtqueue,
    desc_handler: Box<dyn VirtqueueHandler>,
) {
    let avail_ring = virtqueue.avail_ring(mm.as_ref()).unwrap();
    let queue_size = virtqueue.read_queue_size();
    let mut last_available_idx = 0;

    loop {
        select! {
            _ = controller.queue_notify.notified() => {},
            _ = controller.queue_disable.cancelled() => break,
        }

        let mut did_work = false;

        while last_available_idx != avail_ring.idx() {
            // fetch desc from avail ring
            let desc_id = avail_ring.ring(last_available_idx % queue_size);
            last_available_idx += 1;

            let desc_table = virtqueue.desc_table_ref(mm.as_ref()).unwrap();
            let len = desc_handler.handle_desc(&desc_table, desc_id).await;

            // update used ring
            let mut used_ring = virtqueue.used_ring(mm.as_ref()).unwrap();
            let used_idx = used_ring.idx() % queue_size;
            let used_entry = used_ring.ring(used_idx);
            used_entry.id = desc_id as u32;
            used_entry.len = len;
            fence(Ordering::Release);
            used_ring.incr_idx();

            did_work = true;
        }

        if did_work {
            // TODO: if !runtime.disable.is_cancelled()?
            used_buffer_notification.notify_used_buffer();
        }
    }
}
