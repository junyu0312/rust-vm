use std::os::raw::c_void;

use anyhow::bail;
use nix::libc::MAP_ANONYMOUS;
use nix::libc::MAP_FAILED;
use nix::libc::MAP_PRIVATE;
use nix::libc::PROT_READ;
use nix::libc::PROT_WRITE;
use nix::libc::mmap;
use nix::libc::munmap;
use tracing::error;

pub struct MemoryRegion {
    ptr: *mut c_void,
    size: usize,
}

impl MemoryRegion {
    pub fn new(gb: usize) -> anyhow::Result<Self> {
        let size = gb << 30;

        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == MAP_FAILED {
            bail!("Failed to mmap memory region");
        }

        // Add a halt instruction
        unsafe {
            *(ptr as *mut u8) = 0xf4; // x86 HLT instruction
        }

        Ok(MemoryRegion { ptr, size })
    }

    pub fn as_u64(&self) -> u64 {
        self.ptr as u64
    }
}

impl Drop for MemoryRegion {
    fn drop(&mut self) {
        let r = unsafe { munmap(self.ptr, self.size) };

        if r != 0 {
            error!("Failed to munmap memory region");
        }
    }
}
