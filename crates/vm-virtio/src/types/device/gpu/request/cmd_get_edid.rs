use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::device::gpu::request::VirtioGpuCtrlHdr;

#[repr(C)]
#[derive(FromBytes, KnownLayout, Immutable)]
pub struct VirtioGpuGetEdid {
    pub hdr: VirtioGpuCtrlHdr,
    pub scanout: u32,
    padding: u32,
}

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct VirtioGpuRespEdid {
    pub hdr: VirtioGpuCtrlHdr,
    pub size: u32,
    padding: u32,
    pub edid: [u8; 1024],
}
