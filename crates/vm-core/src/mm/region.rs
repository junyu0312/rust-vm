use std::cell::OnceCell;

use crate::mm::Error;
use crate::mm::allocator::Allocator;
use crate::mm::allocator::MemoryContainer;

pub struct MemoryRegion<C> {
    pub gpa: u64,
    pub len: usize,
    // Placeholder: different platforms may use their own allocator for memory.
    pub memory: OnceCell<C>,
}

impl<C> MemoryRegion<C>
where
    C: MemoryContainer,
{
    pub fn placeholder(gpa: u64, len: usize) -> Self {
        MemoryRegion {
            gpa,
            len,
            memory: OnceCell::new(),
        }
    }

    pub fn alloc<A>(&mut self, allocator: &A) -> Result<(), Error>
    where
        A: Allocator<Container = C>,
    {
        if self.memory.get().is_some() {
            return Err(Error::MemoryAlreadyAllocated);
        }

        let memory = allocator.alloc(self.len, None)?;
        let _ = self.memory.set(memory); // already checked

        Ok(())
    }

    pub fn to_hva(&mut self) -> Option<*mut u8> {
        self.memory.get_mut().map(C::to_hva)
    }

    pub fn try_to_hva(&mut self) -> Result<*mut u8, Error> {
        self.to_hva().ok_or(Error::MemoryIsNotAllocated)
    }
}
