use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

pub(crate) mod type0;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct HeaderCommon {
    pub vendor_id: u16,      // 0x00
    pub device_id: u16,      // 0x02
    pub command: u16,        // 0x04
    pub status: u16,         // 0x06
    pub revision_id: u8,     // 0x08
    pub prog_if: u8,         // 0x09
    pub subclass: u8,        // 0x0A
    pub class_code: u8,      // 0x0B
    pub cache_line_size: u8, // 0x0C
    pub latency_timer: u8,   // 0x0D
    pub header_type: u8,     // 0x0E
    pub bist: u8,            // 0x0F
}

pub enum CommonHeaderOffset {
    CapabilityPointer = 0x34,
    CapabilityStart = 0x40,
}
