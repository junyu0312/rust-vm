const OEMID: [u8; 6] = *b"JUNYUZ";
const OEM_TABLE_ID: [u8; 8] = *b"RSVMACPI";
const OEM_REVISION: u32 = 0;
const CREATOR_ID: [u8; 4] = *b"JYVM";
const CREATOR_REVISION: u32 = 0x00000000;

mod r#type;
mod utils;

pub mod dsdt;
pub mod fadt;
pub mod madt;
pub mod rsdp;
pub mod xsdt;
