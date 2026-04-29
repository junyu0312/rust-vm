use vm_snapshot::ops::Pausable;
use vm_snapshot::ops::Snapshotable;

pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;

    fn support_pause(&mut self) -> Option<&mut dyn Pausable> {
        None
    }

    fn support_snapshot(&mut self) -> Option<&mut dyn Snapshotable> {
        None
    }
}
