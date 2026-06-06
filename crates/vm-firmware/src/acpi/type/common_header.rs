use zerocopy::Immutable;
use zerocopy::IntoBytes;

#[derive(Default, Immutable, IntoBytes)]
#[repr(C, packed)]
pub struct CommonHeader {
    pub(crate) signature: [u8; 4],
    pub(crate) length: u32,
    pub(crate) revision: u8,
    pub(crate) checksum: u8,
    pub(crate) oem_id: [u8; 6],
    pub(crate) oem_table_id: [u8; 8],
    pub(crate) oem_revision: u32,
    pub(crate) creator_id: [u8; 4],
    pub(crate) creator_revision: u32,
}
