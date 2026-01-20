use crate::irq::InterruptController;

pub struct HvpGicV3;

impl InterruptController for HvpGicV3 {
    fn trigger_irq(&self, _irq_line: u32, _active: bool) {
        // TODO
    }
}
