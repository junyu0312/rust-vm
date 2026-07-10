use std::collections::BTreeMap;

use vm_virtio::types::device::gpu::request::cmd_resource_create_2d::VirtioGpuFormats;

pub struct VirtioGpuMemBacking {
    pub addr: u64,
    pub length: u32,
}

#[allow(dead_code)]
pub struct VirtioGpuResource {
    pub id: u32,
    pub format: VirtioGpuFormats,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub backing: Vec<VirtioGpuMemBacking>,
    pub buf: Vec<u8>,
}

pub type VirtioGpuResources = BTreeMap<u32, VirtioGpuResource>;
