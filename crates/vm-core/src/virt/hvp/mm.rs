use applevisor::memory::Memory;
use applevisor::vm::VirtualMachineInstance;
use vm_mm::allocator::Allocator;
use vm_mm::allocator::MemoryContainer;
use vm_mm::error::Error;

pub struct MemoryWrapper(pub Memory);

unsafe impl Send for MemoryWrapper {}
unsafe impl Sync for MemoryWrapper {}

impl MemoryContainer for MemoryWrapper {
    fn to_hva(&mut self) -> *mut u8 {
        self.0.host_addr()
    }
}

pub struct HvpAllocator<'a, Gic> {
    pub vm: &'a VirtualMachineInstance<Gic>,
}

impl<'a, Gic> Allocator for HvpAllocator<'a, Gic> {
    type Container = MemoryWrapper;

    fn alloc(&self, len: usize, align: Option<usize>) -> Result<MemoryWrapper, Error> {
        if align.is_some() {
            unimplemented!()
        }

        let mm = self
            .vm
            .memory_create(len)
            .map_err(|_| Error::AllocAnonymousMemoryFailed { len })?;

        Ok(MemoryWrapper(mm))
    }
}
