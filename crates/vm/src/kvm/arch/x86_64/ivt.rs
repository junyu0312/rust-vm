use anyhow::anyhow;

use crate::kvm::vm::KvmVm;

fn ivt_offset(i: usize) -> usize {
    i * 4
}

fn set_ivt(ptr: *mut u8, int: usize, cs: u16, ip: u16) {
    unsafe {
        let offset = ivt_offset(int);
        let ip_ptr = ptr.add(offset) as *mut u16;
        *ip_ptr = ip;
        let cs_ptr = ptr.add(offset + 2) as *mut u16;
        *cs_ptr = cs;
    }
}

impl KvmVm {
    pub fn init_ivt(&mut self) -> anyhow::Result<()> {
        let bios = include_bytes!("../../../../../../bios.bin");

        let memory_regions = self
            .memory_regions
            .get_mut()
            .ok_or_else(|| anyhow!("memory was not initialized"))?;

        let memory_region = memory_regions
            .get_mut_by_gpa(0x0)
            .ok_or_else(|| anyhow!("Failed to get memory region"))?;

        unsafe {
            let dst = memory_region.ptr.add(0x800);
            std::ptr::copy(bios.as_ptr(), dst, bios.len());
        }

        set_ivt(memory_region.ptr, 0x10, 0x0000, 0x0800);
        set_ivt(memory_region.ptr, 0x15, 0x0000, 0x0801);

        Ok(())
    }
}
