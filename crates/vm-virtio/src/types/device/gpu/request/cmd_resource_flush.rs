use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::KnownLayout;

use crate::types::device::gpu::request::VirtioGpuCtrlHdr;
use crate::types::device::gpu::request::virtio_gpu_scanout::VirtioGpuRect;

#[repr(C)]
#[derive(FromBytes, KnownLayout, Immutable)]
pub struct VirtioGpuCmdResourceFlush {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub resource_id: u32,
    padding: u32,
}
