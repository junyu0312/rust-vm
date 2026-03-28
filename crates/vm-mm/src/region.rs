use crate::memory_container::MemoryContainer;

pub struct MemoryRegion {
    pub gpa: u64,
    pub memory: Box<dyn MemoryContainer>,
}

impl MemoryRegion {
    pub fn new(gpa: u64, memory: Box<dyn MemoryContainer>) -> Self {
        MemoryRegion { gpa, memory }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.memory.length()
    }

    pub fn hva(&self) -> *mut u8 {
        self.memory.hva()
    }
}
