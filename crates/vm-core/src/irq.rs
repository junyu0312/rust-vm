pub trait InterruptController: Send + Sync {
    fn trigger_irq(&self, irq_line: u32, active: bool);
}
