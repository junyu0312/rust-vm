use crate::error::Error;
use crate::memory_container::MemoryContainer;

pub mod mmap_allocator;
pub mod std_allocator;

pub trait Allocator {
    type Container: MemoryContainer;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<Self::Container, Error>;
}
