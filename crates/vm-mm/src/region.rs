use crate::allocator::AllocatorKind;
use crate::memory_container::MemoryContainer;

pub mod snapshot;

pub struct MemoryRegion {
    pub gpa: u64,
    pub memory: Box<dyn MemoryContainer>,
}

impl MemoryRegion {
    pub fn new(gpa: u64, memory: Box<dyn MemoryContainer>) -> Self {
        MemoryRegion { gpa, memory }
    }

    pub fn kind(&self) -> AllocatorKind {
        self.memory.kind()
    }

    pub fn align(&self) -> Option<usize> {
        self.memory.align()
    }

    pub fn hva(&self) -> *mut u8 {
        self.memory.hva()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.memory.length()
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.hva(), self.len()) }
    }
}
