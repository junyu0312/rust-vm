use std::fs;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use vm_utils::range_allocator::RangeAllocatorError;

#[derive(Error, Debug)]
pub enum InitrdLoaderError {
    #[error("Failed to read initramfs from {0}")]
    ReadFailed(PathBuf),

    #[error("Copy initrd failed, err: {0}")]
    CopyFailed(#[from] vm_mm::error::Error),

    #[error("Failed to reserve ram for initramfs, err: {0}")]
    ReserveRam(#[from] RangeAllocatorError),
}

pub struct InitrdLoadResult {
    pub initrd_start: u64,
    pub initrd_len: usize,
}

pub struct InitrdLoader {
    initrd: Vec<u8>,
}

impl InitrdLoader {
    pub fn new(path: &Path) -> Result<Self, InitrdLoaderError> {
        let initrd =
            fs::read(path).map_err(|_| InitrdLoaderError::ReadFailed(path.to_path_buf()))?;

        Ok(InitrdLoader { initrd })
    }

    pub fn load(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        addr: u64,
    ) -> Result<InitrdLoadResult, InitrdLoaderError> {
        ram_allocator.reserve(addr, self.initrd.len())?;

        memory.copy_from_slice(addr, &self.initrd)?;

        Ok(InitrdLoadResult {
            initrd_start: addr,
            initrd_len: self.initrd.len(),
        })
    }
}
