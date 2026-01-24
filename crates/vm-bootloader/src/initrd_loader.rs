use std::fs;
use std::path::Path;

use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Read failed")]
    ReadFailed,
    #[error("Copy initrd failed")]
    CopyFailed,
}

pub struct LoadResult {
    pub initrd_start: u64,
    pub initrd_end: u64,
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct InitrdLoader {
    initrd: Vec<u8>,
}

impl InitrdLoader {
    pub fn new(path: &Path) -> Result<Self> {
        let initrd = fs::read(path).map_err(|_| Error::ReadFailed)?;

        Ok(InitrdLoader { initrd })
    }

    pub fn load<C>(&self, addr: u64, memory: &mut MemoryAddressSpace<C>) -> Result<LoadResult>
    where
        C: MemoryContainer,
    {
        memory
            .copy_from_slice(addr, &self.initrd, self.initrd.len())
            .map_err(|_| Error::CopyFailed)?;

        Ok(LoadResult {
            initrd_start: addr,
            initrd_end: addr + self.initrd.len() as u64,
        })
    }
}
