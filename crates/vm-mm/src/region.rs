use crate::memory_container::MemoryContainer;

pub struct MemoryRegion<C> {
    pub gpa: u64,
    pub memory: C,
}

impl<C> MemoryRegion<C>
where
    C: MemoryContainer,
{
    pub fn new(gpa: u64, memory: C) -> Self {
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
