pub mod mmap_allocator;

pub trait MemoryContainer {
    fn to_hva(&self) -> *mut u8;
}

pub trait Allocator {
    type Contrainer: MemoryContainer;

    fn alloc(&self, len: usize, align: Option<usize>) -> anyhow::Result<Self::Contrainer>;
}
