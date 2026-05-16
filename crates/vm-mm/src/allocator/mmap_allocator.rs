use memmap2::MmapMut;

use crate::allocator::Allocator;
use crate::allocator::AllocatorKind;
use crate::error::Error;
use crate::memory_container::MemoryContainer;

pub struct MmapMemoryRegion {
    mmap: MmapMut,
    align: Option<usize>,
}

impl MemoryContainer for MmapMemoryRegion {
    fn kind(&self) -> AllocatorKind {
        AllocatorKind::Mmap
    }

    fn align(&self) -> Option<usize> {
        self.align
    }

    fn hva(&self) -> *mut u8 {
        self.mmap.as_ptr() as *mut u8
    }

    fn length(&self) -> usize {
        self.mmap.len()
    }
}

pub struct MmapAllocator;

impl Allocator for MmapAllocator {
    type Container = MmapMemoryRegion;

    const KIND: AllocatorKind = AllocatorKind::Mmap;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<MmapMemoryRegion, Error> {
        if align.is_some() {
            unimplemented!()
        }

        let mmap = MmapMut::map_anon(len).map_err(|_| Error::AllocAnonymousMemoryFailed { len })?;

        Ok(MmapMemoryRegion { mmap, align })
    }
}
