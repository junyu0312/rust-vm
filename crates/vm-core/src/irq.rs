pub mod arch;

pub trait InterruptController: Send + Sync + 'static {
    fn trigger_irq(&self, irq_line: u32, active: bool);
}
