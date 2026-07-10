use async_trait::async_trait;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::virtqueue::virtq_desc_table::VirtqDescTableRef;

pub struct CursorqHandler;

#[async_trait]
impl VirtqueueHandler for CursorqHandler {
    async fn handle_desc(&self, _desc_ring: &VirtqDescTableRef, _desc_id: u16) -> u32 {
        todo!()
    }
}
