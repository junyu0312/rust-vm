pub struct IrqAllocator(u32);

impl IrqAllocator {
    pub fn new(start: u32) -> Self {
        IrqAllocator(start)
    }

    pub fn alloc(&mut self) -> u32 {
        let irq = self.0;
        self.0 += 1;
        irq
    }
}
