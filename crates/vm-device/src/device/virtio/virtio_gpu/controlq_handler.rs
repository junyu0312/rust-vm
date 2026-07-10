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
use vm_virtio::types::device::gpu::request::cmd_get_edid::VirtioGpuGetEdid;
use vm_virtio::types::device::gpu::request::cmd_get_edid::VirtioGpuRespEdid;
use vm_virtio::types::device::gpu::request::cmd_resource_attach_backing::VirtioGpuMemEntry;
use vm_virtio::types::device::gpu::request::cmd_resource_attach_backing::VirtioGpuResourceAttachBacking;
use vm_virtio::types::device::gpu::request::cmd_resource_create_2d::VirtioGpuFormats;
use vm_virtio::types::device::gpu::request::cmd_resource_create_2d::VirtioGpuResourceCreate2D;
use vm_virtio::types::device::gpu::request::cmd_resource_detach_baking::VirtioGpuResourceDetachBacking;
use vm_virtio::types::device::gpu::request::cmd_resource_flush::VirtioGpuCmdResourceFlush;
use vm_virtio::types::device::gpu::request::cmd_resource_unref::VirtioGpuResourceUnref;
use vm_virtio::types::device::gpu::request::cmd_set_scanout::VirtioGpuSetScanout;
use vm_virtio::types::device::gpu::request::cmd_transfer_to_host_2d::VirtioGpuTransferToHost2D;
use vm_virtio::virtqueue::virtq_desc_table::VirtqDescTableRef;
use zerocopy::IntoBytes;

use crate::device::virtio::virtio_gpu::resource::VirtioGpuMemBacking;
use crate::device::virtio::virtio_gpu::resource::VirtioGpuResource;
use crate::device::virtio::virtio_gpu::resource::VirtioGpuResources;
use crate::device::virtio::virtio_gpu::scanout::Scanout;

fn copy_from_iov(
    memory: &MemoryAddressSpace,
    iovs: &[VirtioGpuMemBacking],
    src_offset: u64,
    mut dst: &mut [u8],
) -> bool {
    let mut offset = src_offset;

    for iov in iovs {
        if offset >= iov.length as u64 {
            offset -= iov.length as u64;
            continue;
        }

        let len = (iov.length as usize - offset as usize).min(dst.len());

        memory
            .copy_to_slice(iov.addr + offset, &mut dst[..len])
            .unwrap();

        dst = &mut dst[len..];

        if dst.is_empty() {
            return true;
        }

        offset = 0;
    }

    false
}

fn copy_from_framebuffer_to_resource(
    memory: &MemoryAddressSpace,
    resource: &mut VirtioGpuResource,
    src_offset: u64,
    dst_offset: u32,
    len: usize,
) {
    let dst = &mut resource.buf[dst_offset as usize..dst_offset as usize + len];

    let is_empty = copy_from_iov(memory, &resource.backing, src_offset, dst);

    assert!(is_empty);
}

pub struct ControlqHandler {
    pub resource: Arc<Mutex<VirtioGpuResources>>,
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

    async fn handle_resource_create_2d(
        &self,
        cmd: &VirtioGpuResourceCreate2D,
        resp: &mut VirtioGpuCtrlHdr,
    ) {
        let mut resources = self.resource.lock().await;

        let Some(format) = VirtioGpuFormats::from_repr(cmd.format) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER as u32;

            return;
        };

        let Some(stride) = (cmd.width as usize).checked_mul(format.bpp()) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER as u32;

            return;
        };

        let Some(len) = (cmd.height as usize).checked_mul(stride) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER as u32;

            return;
        };

        let buf = vec![0; len];

        resources.insert(
            cmd.resource_id,
            VirtioGpuResource {
                id: cmd.resource_id,
                format,
                width: cmd.width,
                height: cmd.height,
                stride,
                buf,
                backing: Default::default(),
            },
        );

        resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
    }

    async fn handle_resource_unref(
        &self,
        cmd: &VirtioGpuResourceUnref,
        resp: &mut VirtioGpuCtrlHdr,
    ) {
        let mut resource = self.resource.lock().await;

        match resource.remove(&cmd.resource_id) {
            Some(_) => {
                resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
            }
            None => {
                resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID as u32;
            }
        }
    }

    async fn handle_set_scanout(&self, cmd: &VirtioGpuSetScanout, resp: &mut VirtioGpuCtrlHdr) {
        if cmd.resource_id != 0 {
            let resources = self.resource.lock().await;
            let Some(_) = resources.get(&cmd.resource_id) else {
                resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID as u32;

                return;
            };
        }

        let mut scanouts = self.scanouts.lock().await;

        let Some(scanout) = scanouts.get_mut(cmd.scanout_id as usize) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_SCANOUT_ID as u32;

            return;
        };

        scanout.resource = cmd.resource_id;
        scanout.rect = Some(cmd.r.clone());

        resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
    }

    async fn handle_resource_flush(
        &self,
        cmd: &VirtioGpuCmdResourceFlush,
        resp: &mut VirtioGpuCtrlHdr,
    ) {
        let mut resources = self.resource.lock().await;
        let Some(_) = resources.get_mut(&cmd.resource_id) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID as u32;

            return;
        };

        // TODO: Implement the actual flush logic here. For now, we just acknowledge the command.

        resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
    }

    async fn handle_transfer_to_host_2d(
        &self,
        cmd: &VirtioGpuTransferToHost2D,
        resp: &mut VirtioGpuCtrlHdr,
    ) {
        let mut resources = self.resource.lock().await;
        let Some(resource) = resources.get_mut(&cmd.resource_id) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID as u32;

            return;
        };

        if cmd.r.x > resource.width
            || cmd.r.width > resource.width
            || cmd.r.x + cmd.r.width > resource.width
            || cmd.r.y > resource.height
            || cmd.r.height > resource.height
            || cmd.r.y + cmd.r.height > resource.height
        {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER as u32;

            return;
        }

        let bpp = resource.format.bpp() as u32;
        let rect = &cmd.r;
        {
            let len = rect.width * bpp;
            for row in 0..rect.height {
                let src_offset = cmd.offset + (row * resource.stride as u32) as u64;
                let dst_offset = (rect.y + row) * resource.stride as u32 + rect.x * bpp;

                copy_from_framebuffer_to_resource(
                    &self.memory,
                    resource,
                    src_offset,
                    dst_offset,
                    len as usize,
                );
            }
        }

        resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
    }

    async fn handle_resource_attach_backing(
        &self,
        cmd: &VirtioGpuResourceAttachBacking,
        entries: Vec<&VirtioGpuMemEntry>,
        resp: &mut VirtioGpuCtrlHdr,
    ) {
        let resource_id = cmd.resource_id;

        let mut resources = self.resource.lock().await;
        let Some(resource) = resources.get_mut(&resource_id) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID as u32;

            return;
        };

        if !resource.backing.is_empty() {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_UNSPEC as u32;

            return;
        }

        for entry in entries {
            resource.backing.push(VirtioGpuMemBacking {
                addr: entry.addr,
                length: entry.length,
            });
        }

        resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
    }

    async fn handle_resource_detach_backing(
        &self,
        cmd: &VirtioGpuResourceDetachBacking,
        resp: &mut VirtioGpuCtrlHdr,
    ) {
        let mut resources = self.resource.lock().await;
        let Some(resource) = resources.get_mut(&cmd.resource_id) else {
            resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID as u32;

            return;
        };

        resource.backing.clear();

        resp.r#type = VirtioGpuCtrlType::VIRTIO_GPU_RESP_OK_NODATA as u32;
    }

    fn handle_get_edid(&self, _cmd: &VirtioGpuGetEdid, _resp: &mut VirtioGpuRespEdid) {
        todo!()
    }

    async fn handle_command(
        &self,
        desc_ring: &VirtqDescTableRef,
        desc_id: u16,
    ) -> Result<u32, VirtioError> {
        let desc_entry = desc_ring.get(desc_id);
        let command = desc_entry.as_ref::<VirtioGpuCtrlHdr>(&self.memory)?;

        let ctrl_type = VirtioGpuCtrlType::from_repr(command.r#type)
            .ok_or(VirtioGpuError::UnknownCtrlType(command.r#type))?;

        match ctrl_type {
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_DISPLAY_INFO => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let response = chain[1].as_mut::<VirtioGpuRespDisplayInfo>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_get_display_info(response).await;

                Ok(size_of::<VirtioGpuRespDisplayInfo>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_CREATE_2D => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuResourceCreate2D>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_resource_create_2d(cmd, response).await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_UNREF => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuResourceUnref>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_resource_unref(cmd, response).await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_SET_SCANOUT => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuSetScanout>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_set_scanout(cmd, response).await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_FLUSH => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuCmdResourceFlush>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_resource_flush(cmd, response).await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuTransferToHost2D>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_transfer_to_host_2d(cmd, response).await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 3);

                let cmd = chain[0].as_ref::<VirtioGpuResourceAttachBacking>(&self.memory)?;
                let entries = chain[1]
                    .as_slice::<VirtioGpuMemEntry>(&self.memory, cmd.nr_entries as usize)?;
                let response = chain[2].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_resource_attach_backing(cmd, entries, response)
                    .await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_RESOURCE_DETACH_BACKING => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuResourceDetachBacking>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuCtrlHdr>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_resource_detach_backing(cmd, response).await;

                Ok(size_of::<VirtioGpuCtrlHdr>().try_into().unwrap())
            }
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_CAPSET_INFO => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_CAPSET => todo!(),
            VirtioGpuCtrlType::VIRTIO_GPU_CMD_GET_EDID => {
                let chain = desc_ring.get_chain(desc_id);
                assert_eq!(chain.len(), 2);

                let cmd = chain[0].as_ref::<VirtioGpuGetEdid>(&self.memory)?;
                let response = chain[1].as_mut::<VirtioGpuRespEdid>(&self.memory)?;
                response.as_mut_bytes().fill(0);

                self.handle_get_edid(cmd, response);

                Ok(size_of::<VirtioGpuRespEdid>().try_into().unwrap())
            }
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
            _ => Err(VirtioError::VirtioGpu(VirtioGpuError::InvalidCommand(
                ctrl_type,
            ))),
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
