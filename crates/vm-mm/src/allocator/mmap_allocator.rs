use memmap2::MmapMut;

use crate::allocator::Allocator;
use crate::error::Error;
use crate::memory_container::MemoryContainer;

impl MemoryContainer for MmapMut {
    fn hva(&self) -> *mut u8 {
        self.as_ptr() as *mut u8
    }

    fn length(&self) -> usize {
        self.len()
    }
}

pub struct MmapAllocator;

impl Allocator for MmapAllocator {
    type Container = MmapMut;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<MmapMut, Error> {
        if align.is_some() {
            unimplemented!()
        }

        MmapMut::map_anon(len).map_err(|_| Error::AllocAnonymousMemoryFailed { len })
    }
}
