use serde::Deserialize;
use serde::Serialize;

use crate::allocator::AllocatorKind;

#[derive(Serialize, Deserialize)]
pub struct MemoryRegionSnapshot {
    pub gpa: u64,
    pub align: Option<usize>,
    pub kind: AllocatorKind,
    pub buf: Vec<u8>,
}
