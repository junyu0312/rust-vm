use vm_snapshot::ops::Pausable;
use vm_snapshot::ops::Snapshotable;

use crate::device::error::DeviceSnapshotError;

pub mod error;
pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;

    fn support_pause(&self) -> Option<&dyn Pausable<Error = DeviceSnapshotError>> {
        None
    }

    fn support_snapshot(&self) -> Option<&dyn Snapshotable<Error = DeviceSnapshotError>> {
        None
    }
}
