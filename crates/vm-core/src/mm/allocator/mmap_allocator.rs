use std::ffi::c_void;

use anyhow::bail;
use nix::libc::{MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap, munmap};

use crate::mm::allocator::{Allocator, MemoryContainer};

pub struct MmapMemory {
    ptr: *mut u8,
    len: usize,
}

impl Drop for MmapMemory {
    fn drop(&mut self) {
        let _ = unsafe { munmap(self.ptr as *mut c_void, self.len) };
    }
}

impl MemoryContainer for MmapMemory {
    fn to_hva(&self) -> *mut u8 {
        self.ptr
    }
}

pub struct MmapAllocator;

impl Allocator for MmapAllocator {
    type Contrainer = MmapMemory;

    fn alloc(&self, len: usize, align: Option<usize>) -> anyhow::Result<MmapMemory> {
        if align.is_some() {
            unimplemented!()
        }

        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == MAP_FAILED {
            bail!("Failed to mmap memory region");
        }

        Ok(MmapMemory {
            ptr: ptr as *mut u8,
            len,
        })
    }
}
