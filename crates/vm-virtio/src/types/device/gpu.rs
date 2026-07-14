use bitflags::bitflags;
use strum_macros::FromRepr;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

pub mod error;
pub mod request;

bitflags! {
    pub struct VirtioGpuFeatures: u32 {
        const Virgl = 1 << 0;
        const Edid = 1 << 1;
        const ResourceUuid = 1 << 2;
        const ResourceBlob = 1 << 3;
        const ContextInit = 1 << 4;
        const BlobAlignment = 1 << 5;
    }
}

/// Display configuration has changed. The driver SHOULD use the
/// VIR-TIO_GPU_CMD_GET_DISPLAY_INFO command to fetch the information
/// from the device. In case EDID support is negotiated
/// (VIRTIO_GPU_F_EDID feature flag) the device SHOULD also fetch the
/// updated EDID blobs using the VIRTIO_GPU_CMD_GET_EDID command.
pub const VIRTIO_GPU_EVENT_DISPLAY: u32 = 1 << 0;

#[repr(C)]
#[derive(Default, FromBytes, IntoBytes, Immutable)]
pub struct VirtioGpuConfig {
    /// signals pending events to the driver. The driver MUST NOT write to this field.
    pub events_read: u32,
    /// clears pending events in the device. Writing a ’1’ into a bit will clear the corresponding bit in
    /// events_read, mimicking write-to-clear behavior.
    pub events_clear: u32,
    /// specifies the maximum number of scanouts supported by the device. Minimum value is 1,
    /// maximum value is 16.
    pub num_scanouts: u32,
    ///  specifies the maximum number of capability sets supported by the device. The minimum
    /// value is zero.
    pub num_capsets: u32,
    /// specifies the minimal alignment, in bytes, required by the device for resource blobs. The
    /// value is a power of two. Minimum value is 1, maximum value is 4294967296.
    pub blob_alignment: u32,
}

#[derive(FromRepr)]
pub enum VirtioGpuConfigOffset {
    EventsRead = 0x00,
    EventsClear = 0x04,
    NumScanouts = 0x08,
    NumCapset = 0x0c,
    BlobAlignments = 0x10,
}
