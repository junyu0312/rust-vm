use memmap2::MmapMut;

use crate::allocator::Allocator;
use crate::error::Error;

mod container {
    use memmap2::MmapMut;

    use crate::allocator::MemoryContainer;

    impl MemoryContainer for MmapMut {
        fn to_hva(&mut self) -> *mut u8 {
            self.as_mut_ptr()
        }
    }
}

pub struct MmapAllocator;

impl Allocator for MmapAllocator {
    type Container = MmapMut;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<MmapMut, Error> {
        if align.is_some() {
            unimplemented!()
        }

        let mmap = MmapMut::map_anon(len).map_err(|_| Error::AllocAnonymousMemoryFailed { len })?;

        Ok(mmap)
    }
}
