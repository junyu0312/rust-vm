use crate::allocator::MemoryContainer;

pub struct MemoryRegion<C> {
    pub gpa: u64,
    pub len: usize,
    pub memory: C,
}

impl<C> MemoryRegion<C>
where
    C: MemoryContainer,
{
    pub fn new(gpa: u64, len: usize, memory: C) -> Self {
        MemoryRegion { gpa, len, memory }
    }

    pub fn to_hva(&mut self) -> *mut u8 {
        self.memory.to_hva()
    }
}
