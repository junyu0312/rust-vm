use vm_core::irq::InterruptController;

pub mod atkbd;
pub mod psmouse;

pub trait Ps2Device {
    const IRQ: u32;

    fn trigger_irq(&mut self, irq: &dyn InterruptController, active: bool) {
        irq.trigger_irq(Self::IRQ, active);
    }

    fn output_buffer_is_empty(&self) -> bool;

    fn output_buffer_is_full(&self) -> bool;

    fn try_push_output_buffer(&mut self, value: u8) -> Result<(), u8>;

    fn pop_output_buffer(&mut self) -> Option<u8>;

    fn handle_command(&mut self, cmd: u8);
}
