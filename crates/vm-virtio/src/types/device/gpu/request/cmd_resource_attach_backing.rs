use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::KnownLayout;

use crate::types::device::gpu::request::VirtioGpuCtrlHdr;

#[repr(C)]
#[derive(FromBytes, KnownLayout, Immutable)]
pub struct VirtioGpuResourceAttachBacking {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub nr_entries: u32,
}

#[repr(C)]
#[derive(FromBytes, KnownLayout, Immutable)]
pub struct VirtioGpuMemEntry {
    pub addr: u64,
    pub length: u32,
    padding: u32,
}
