use anyhow::anyhow;

use crate::kvm::vm::KvmVm;

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct IvEntry {
    ip: u16,
    cs: u16,
}

impl From<u32> for IvEntry {
    fn from(addr: u32) -> Self {
        IvEntry {
            ip: (addr & 0xffff) as u16,
            cs: (addr >> 16) as u16,
        }
    }
}

#[repr(C)]
struct InterruptVectorTable {
    entries: [IvEntry; 256],
}

impl Default for InterruptVectorTable {
    fn default() -> Self {
        Self {
            entries: [Default::default(); 256],
        }
    }
}

impl InterruptVectorTable {
    fn set_entry(&mut self, index: u32, addr: u32) {
        self.entries[index as usize] = IvEntry::from(addr);
    }

    fn len(&self) -> usize {
        self.entries.len() * 4
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as *const u8,
                core::mem::size_of::<InterruptVectorTable>(),
            )
        }
    }
}

#[derive(Default)]
pub struct Bios;

impl Bios {
    #[allow(warnings)]
    pub fn init(&self, vm: &mut KvmVm) -> anyhow::Result<()> {
        let mut ivt = InterruptVectorTable::default();
        ivt.set_entry(0x10, todo!());
        ivt.set_entry(0x15, todo!());

        let memory_region = vm
            .memory_regions
            .get_mut()
            .ok_or_else(|| anyhow!("Memory is not initialized"))?;
        memory_region.copy_from_slice(0, ivt.as_bytes(), ivt.len())?;
        Ok(())
    }
}
