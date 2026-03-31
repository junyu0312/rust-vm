use std::alloc::Layout;
use std::alloc::alloc_zeroed;

use crate::allocator::Allocator;
use crate::error::Error;
use crate::memory_container::MemoryContainer;

pub struct StdMemoryRegion {
    addr: *mut u8,
    len: usize,
    layout: Layout,
}

unsafe impl Send for StdMemoryRegion {}
unsafe impl Sync for StdMemoryRegion {}

impl Drop for StdMemoryRegion {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.addr, self.layout) };
    }
}

impl MemoryContainer for StdMemoryRegion {
    fn hva(&self) -> *mut u8 {
        self.addr
    }

    fn length(&self) -> usize {
        self.len
    }
}

pub struct StdAllocator;

impl Allocator for StdAllocator {
    type Container = StdMemoryRegion;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<StdMemoryRegion, Error> {
        let layout = if let Some(align) = align {
            Layout::from_size_align(len, align)
        } else {
            Layout::array::<u8>(len)
        }
        .map_err(|_| Error::AllocAnonymousMemoryFailed { len })?;

        let addr = unsafe { alloc_zeroed(layout) };

        Ok(StdMemoryRegion { addr, len, layout })
    }
}
