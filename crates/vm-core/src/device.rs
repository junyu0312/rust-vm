use vm_snapshot::ops::Pausable;
use vm_snapshot::ops::Snapshotable;

use crate::device::error::DeviceError;

pub mod error;
pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;

    fn support_pause(&mut self) -> Option<&mut dyn Pausable<Error = DeviceError>> {
        None
    }

    fn support_snapshot(&mut self) -> Option<&mut dyn Snapshotable<Error = DeviceError>> {
        None
    }
}
