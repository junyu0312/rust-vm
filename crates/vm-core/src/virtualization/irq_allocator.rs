use crate::virtualization::vm::error::VmError;

pub struct IrqAllocator {
    max: u32,
    current: u32,
}

impl IrqAllocator {
    pub fn new(min: u32, max: u32) -> Self {
        IrqAllocator { max, current: min }
    }

    pub fn alloc(&mut self) -> Result<u32, VmError> {
        if self.current > self.max {
            return Err(VmError::AllocIrq);
        }

        let alloc = self.current;
        self.current = self.current.checked_add(1).ok_or(VmError::AllocIrq)?;

        Ok(alloc)
    }
}
