use std::cell::OnceCell;

use anyhow::anyhow;
use anyhow::bail;

use crate::mm::allocator::Allocator;
use crate::mm::allocator::MemoryContainer;

pub struct MemoryRegion<C> {
    pub gpa: u64,
    pub len: usize,
    pub memory: OnceCell<C>,
}

impl<C> MemoryRegion<C>
where
    C: MemoryContainer,
{
    pub fn new(gpa: u64, len: usize) -> anyhow::Result<Self> {
        Ok(MemoryRegion {
            gpa,
            len,
            memory: OnceCell::new(),
        })
    }

    pub fn alloc<A>(&mut self, allocator: &A) -> anyhow::Result<()>
    where
        A: Allocator<Contrainer = C>,
    {
        if self.memory.get().is_some() {
            bail!(anyhow!("memory is already initialized"));
        }

        let memory = allocator.alloc(self.len, None)?;
        self.memory
            .set(memory)
            .map_err(|_| anyhow!("memory is already initialized"))?;

        Ok(())
    }

    pub fn to_hva(&self) -> Option<*mut u8> {
        self.memory.get().map(|m| m.to_hva())
    }
}
