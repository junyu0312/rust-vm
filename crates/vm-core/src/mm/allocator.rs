use crate::mm::Error;

pub mod mmap_allocator;

pub trait MemoryContainer: Send + Sync + 'static {
    fn to_hva(&mut self) -> *mut u8;
}

pub trait Allocator {
    type Container: MemoryContainer;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<Self::Container, Error>;
}
