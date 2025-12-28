use std::fs;
use std::path::Path;

use anyhow::anyhow;

use crate::kvm::vm::KvmVm;

const KERNEL_LOAD_ADDR: usize = 0x90000;

impl KvmVm {
    pub fn init_kernel(&mut self, kernel: &Path) -> anyhow::Result<()> {
        let data = fs::read(kernel)?;

        let memory_regions = self
            .memory_regions
            .get_mut()
            .ok_or_else(|| anyhow!("memory was not initialized"))?;

        let memory_region = memory_regions
            .get_mut_by_gpa(0x90000)
            .ok_or_else(|| anyhow!("Failed to get memory region"))?;

        unsafe {
            let dst = memory_region.ptr.add(KERNEL_LOAD_ADDR);
            std::ptr::copy(data.as_ptr(), dst, data.len());
        }

        let boot_flag = unsafe { *(memory_region.ptr.add(KERNEL_LOAD_ADDR + 0x1fe) as *const u16) };
        assert_eq!(boot_flag, 0xAA55);

        Ok(())
    }
}
