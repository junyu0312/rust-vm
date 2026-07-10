use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::error;
use vm_mm::manager::MemoryAddressSpace;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::result::VirtioError;
use vm_virtio::types::device::gpu::error::VirtioGpuError;
use vm_virtio::types::device::gpu::request::VirtioGpuCtrlHdr;
use vm_virtio::types::device::gpu::request::VirtioGpuCtrlType;
use vm_virtio::types::device::gpu::request::cmd_get_display_info::VirtioGpuRespDisplayInfo;
use vm_virtio::virtqueue::virtq_desc_table::VirtqDescTableRef;
use zerocopy::IntoBytes;

use crate::device::virtio::virtio_gpu::scanout::Scanout;

pub struct ControlqHandler {
    pub scanouts: Arc<Mutex<Vec<Scanout>>>,
    pub memory: Arc<MemoryAddressSpace>,
}

impl ControlqHandler {
    async fn handle_get_display_info(&self, resp: &mut VirtioGpuRespDisplayInfo) {
        let scanouts = self.scanouts.lock().await;

        resp.hdr.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_DISPLAY_INFO as u32;

        for (index, scanout) in resp.pmodes.iter_mut().enumerate() {
            if let Some(s) = scanouts.get(index) {
                scanout.r.x = 0;
                scanout.r.y = 0;
                scanout.r.width = s.width;
                scanout.r.height = s.height;
                scanout.enabled = 1;
            }
        }
    }

    async fn handle_command(
        &self,
        desc_ring: &VirtqDescTableRef,
        desc_id: u16,
    ) -> Result<u32, VirtioError> {
        let desc_entry = desc_ring.get(desc_id);
        let command = desc_entry.as_ref::<VirtioGpuCtrlHdr>(&self.memory)?;

        let ctrl_type = VirtioGpuCtrlType::from_repr(command.r#type)
            .ok_or_else(|| VirtioGpuError::UnknownCtrlType(command.r#type))?;

        match ctrl_type {
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_DISPLAY_INFO => {
                let chain = desc_ring.get_chain(desc_id);
                let response = chain[1].as_mut::<VirtioGpuRespDisplayInfo>(&self.memory)?;
                response.as_mut_bytes().fill(0);
                self.handle_get_display_info(response).await;

                Ok(size_of::<VirtioGpuRespDisplayInfo>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_CREATE_2D => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_UNREF => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_SET_SCANOUT => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_FLUSH => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_DETACH_BACKING => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_CAPSET_INFO => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_CAPSET => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_EDID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_ASSIGN_UUID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_CREATE_BLOB => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_SET_SCANOUT_BLOB => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_CTX_CREATE => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_CTX_DESTROY => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_CTX_ATTACH_RESOURCE => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_CTX_DETACH_RESOURCE => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_CREATE_3D => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_TRANSFER_TO_HOST_3D => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_TRANSFER_FROM_HOST_3D => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_SUBMIT_3D => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_MAP_BLOB => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_UNMAP_BLOB => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_UPDATE_CURSOR => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_MOVE_CURSOR => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_DISPLAY_INFO => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_CAPSET_INFO => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_CAPSET => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_EDID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_RESOURCE_UUID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_MAP_INFO => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_UNSPEC => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_OUT_OF_MEMORY => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_SCANOUT_ID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_CONTEXT_ID => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER => todo!(),
        }
    }
}

#[async_trait]
impl VirtqueueHandler for ControlqHandler {
    async fn handle_desc(&self, desc_ring: &VirtqDescTableRef, desc_id: u16) -> u32 {
        match self.handle_command(desc_ring, desc_id).await {
            Ok(r) => r,
            Err(err) => {
                error!(?err, "Failed to handle virtio-gpu command");
                0
            }
        }
    }
}
