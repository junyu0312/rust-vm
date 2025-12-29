use std::path::Path;

use anyhow::anyhow;
use vm_linux::arch::x86_64::bzimage::BzImage;

use crate::kvm::vm::KvmVm;

pub const KERNEL_LOAD_ADDR: usize = 0x90000;

impl KvmVm {
    pub fn init_kernel(&mut self, kernel: &Path) -> anyhow::Result<()> {
        let mut bzimage = BzImage::read(kernel)?;

        assert_eq!(bzimage.get_boot_flag()?, 0xAA55);

        bzimage.set_heap_end_ptr(0x9800 - 0x200)?;
        bzimage.set_loadflags(bzimage.get_loadflags()? | 0x80)?;
        bzimage.set_cmd_line_ptr(KERNEL_LOAD_ADDR as u32 + 0x9800)?;
        bzimage.set_cmdline(b"earlyprintk=serial,console=ttyS0\0", 0x9800)?;

        let memory_regions = self
            .memory_regions
            .get_mut()
            .ok_or_else(|| anyhow!("memory was not initialized"))?;

        let memory_region = memory_regions
            .get_mut_by_gpa(0x90000)
            .ok_or_else(|| anyhow!("Failed to get memory region"))?;

        unsafe {
            let dst = memory_region.ptr.add(KERNEL_LOAD_ADDR);
            std::ptr::copy(bzimage.as_ptr(), dst, bzimage.len());
        }

        Ok(())
    }
}
