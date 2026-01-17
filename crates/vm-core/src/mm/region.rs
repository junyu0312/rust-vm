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
    pub gpa: usize,
    pub ptr: *mut u8,
    pub len: usize,
}

impl MemoryRegion {
    pub fn new(gpa: usize, len: usize) -> anyhow::Result<Self> {
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

        let ptr = ptr as *mut u8;

        Ok(MemoryRegion { gpa, ptr, len })
    }

    pub fn as_u64(&self) -> u64 {
        self.ptr as u64
    }
}

impl Drop for MemoryRegion {
    fn drop(&mut self) {
        let r = unsafe { munmap(self.ptr as *mut c_void, self.len) };

        if r != 0 {
            error!("Failed to munmap memory region");
        }
    }
}
