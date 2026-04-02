pub struct IrqAllocation(u32);

impl IrqAllocation {
    pub fn new(start: u32) -> Self {
        IrqAllocation(start)
    }

    pub fn alloc(&mut self) -> u32 {
        let irq = self.0;
        self.0 += 1;
        irq
    }
}
