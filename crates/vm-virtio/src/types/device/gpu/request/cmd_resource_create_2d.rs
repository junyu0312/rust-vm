use strum_macros::FromRepr;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::KnownLayout;

use crate::types::device::gpu::request::VirtioGpuCtrlHdr;

#[repr(u32)]
#[derive(FromRepr)]
pub enum VirtioGpuFormats {
    B8G8R8A8 = 1,
    B8G8R8X8 = 2,
    A8R8G8B8 = 3,
    X8R8G8B8 = 4,
    R8G8B8A8 = 67,
    X8B8G8R8 = 68,
    A8B8G8R8 = 121,
    R8G8B8X8 = 134,
}

impl VirtioGpuFormats {
    pub fn bpp(&self) -> usize {
        match self {
            VirtioGpuFormats::B8G8R8A8 => 4,
            VirtioGpuFormats::B8G8R8X8 => 4,
            VirtioGpuFormats::A8R8G8B8 => 4,
            VirtioGpuFormats::X8R8G8B8 => 4,
            VirtioGpuFormats::R8G8B8A8 => 4,
            VirtioGpuFormats::X8B8G8R8 => 4,
            VirtioGpuFormats::A8B8G8R8 => 4,
            VirtioGpuFormats::R8G8B8X8 => 4,
        }
    }
}

#[repr(C)]
#[derive(FromBytes, KnownLayout, Immutable)]
pub struct VirtioGpuResourceCreate2D {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}
