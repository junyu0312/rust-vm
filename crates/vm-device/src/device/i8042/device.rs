use std::collections::VecDeque;

use crate::device::irq::InterruptController;

const BUFFER_SIZE: usize = 16;

#[derive(Default)]
pub struct Ps2Device<const IRQ: u32> {
    output: VecDeque<u8>,
}

impl<const IRQ: u32> Ps2Device<IRQ> {
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.output.len() == BUFFER_SIZE
    }

    pub fn pop_front(&mut self) -> Option<u8> {
        self.output.pop_front()
    }

    pub fn try_push_back(&mut self, value: u8) -> Result<(), u8> {
        if self.output.len() >= BUFFER_SIZE {
            return Err(value);
        }

        self.output.push_back(value);

        Ok(())
    }

    pub fn trigger_irq(&self, irq_controller: &dyn InterruptController, active: bool) {
        irq_controller.trigger_irq(IRQ, active);
    }
}
