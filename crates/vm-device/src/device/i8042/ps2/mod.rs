use std::collections::VecDeque;

use crate::device::irq::InterruptController;

pub mod psmouse;

pub trait Ps2DeviceT {
    const IRQ: u32;
    const OUTPUT_BUFFER_SIZE: usize;

    fn trigger_irq(&mut self, irq: &dyn InterruptController, active: bool) {
        irq.trigger_irq(Self::IRQ, active);
    }

    fn output_buffer_is_empty(&self) -> bool;

    fn output_buffer_is_full(&self) -> bool;

    fn try_push_output_buffer(&mut self, value: u8) -> Result<(), u8>;

    fn pop_output_buffer(&mut self) -> Option<u8>;
}

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
