use std::fs;
use std::path::Path;

use thiserror::Error;
use vm_mm::manager::MemoryAddressSpace;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Read failed")]
    ReadFailed,
    #[error("Copy initrd failed")]
    CopyFailed,
}

pub struct LoadResult {
    pub initrd_start: u64,
    pub initrd_len: usize,
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

    pub fn load(&self, addr: u64, memory: &MemoryAddressSpace) -> Result<LoadResult> {
        memory
            .copy_from_slice(addr, &self.initrd)
            .map_err(|_| Error::CopyFailed)?;

        Ok(LoadResult {
            initrd_start: addr,
            initrd_len: self.initrd.len(),
        })
    }
}
