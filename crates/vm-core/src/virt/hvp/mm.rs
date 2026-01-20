use applevisor::{
    memory::Memory,
    vm::{GicDisabled, VirtualMachineInstance},
};

use crate::mm::allocator::{Allocator, MemoryContainer};

impl MemoryContainer for Memory {
    fn to_hva(&self) -> *mut u8 {
        self.host_addr()
    }
}

pub struct HvpAllocator<'a> {
    pub vm: &'a VirtualMachineInstance<GicDisabled>,
}

impl<'a> Allocator for HvpAllocator<'a> {
    type Contrainer = Memory;

    fn alloc(&self, len: usize, align: Option<usize>) -> anyhow::Result<Memory> {
        if align.is_some() {
            unimplemented!()
        }

        let mm = self.vm.memory_create(len)?;

        Ok(mm)
    }
}
