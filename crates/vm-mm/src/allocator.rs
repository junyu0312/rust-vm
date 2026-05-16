use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;
use crate::memory_container::MemoryContainer;

pub mod mmap_allocator;
pub mod std_allocator;

#[derive(Serialize, Deserialize)]
pub enum AllocatorKind {
    Mmap,
    Std,
}

pub trait Allocator {
    type Container: MemoryContainer;

    const KIND: AllocatorKind;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<Self::Container, Error>;
}
