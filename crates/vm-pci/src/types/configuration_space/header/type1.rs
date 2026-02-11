use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::types::configuration_space::header::HeaderCommon;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct Type1Header {
    pub common: HeaderCommon,            // 16B
    pub bar0: u32,                       // 0x10
    pub bar1: u32,                       // 0x14
    pub primary_bus_number: u8,          // 0x18
    pub secondary_bus_number: u8,        // 0x19
    pub subordinate_bus_number: u8,      // 0x1A
    pub secondary_latency_timer: u8,     // 0x1B
    pub io_base: u8,                     // 0x1C
    pub io_limit: u8,                    // 0x1D
    pub secondary_status: u16,           // 0x1E
    pub memory_base: u16,                // 0x20
    pub memory_limit: u16,               // 0x22
    pub prefetchable_memory_base: u16,   // 0x24
    pub prefetchable_memory_limit: u16,  // 0x26
    pub prefetchable_base_upper32: u32,  // 0x28
    pub prefetchable_limit_upper32: u32, // 0x2C
    pub io_base_upper16: u16,            // 0x30
    pub io_limit_upper16: u16,           // 0x32
    pub capabilities_ptr: u8,            // 0x34
    reserved1: [u8; 3],                  // 0x35~0x37
    pub expansion_rom_base_address: u32, // 0x38
    pub interrupt_line: u8,              // 0x3C
    pub interrupt_pin: u8,               // 0x3D
    pub bridge_control: u16,             // 0x3E
}
