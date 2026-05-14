use vm_snapshot::ops::Pausable;
use vm_snapshot::ops::Snapshotable;

use crate::device::error::DeviceSnapshotError;

pub mod error;
pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;

    fn support_pause(&self) -> Option<&mut dyn Pausable<Error = DeviceSnapshotError>> {
        None
    }

    fn support_snapshot(&self) -> Option<&mut dyn Snapshotable<Error = DeviceSnapshotError>> {
        None
    }
}
