use std::io::Write;
use std::os::raw::c_void;
use std::slice;

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
    ptr: *mut u8,
    mem_size: usize,
}

impl MemoryRegion {
    pub fn new(gb: usize) -> anyhow::Result<Self> {
        let mem_size = gb << 30;

        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                mem_size,
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

        let x86_code = [0xf4 /* hlt */];
        unsafe {
            let mut slice = slice::from_raw_parts_mut(ptr, mem_size);
            slice.write_all(&x86_code)?;
        }

        Ok(MemoryRegion { ptr, mem_size })
    }

    pub fn as_u64(&self) -> u64 {
        self.ptr as u64
    }
}

impl Drop for MemoryRegion {
    fn drop(&mut self) {
        let r = unsafe { munmap(self.ptr as *mut c_void, self.mem_size) };

        if r != 0 {
            error!("Failed to munmap memory region");
        }
    }
}
