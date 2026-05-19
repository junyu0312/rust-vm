use vm_snapshot::ops::{LoadSnapshot, Pausable, SaveSnapshot};

use crate::device::error::DeviceSnapshotError;

pub mod error;
pub mod mmio;
pub mod pio;

pub trait Device: Send + Sync {
    fn name(&self) -> String;

    fn support_pause(&self) -> Option<&dyn Pausable<Error = DeviceSnapshotError>> {
        None
    }

    fn support_save_snapshot(&self) -> Option<&dyn SaveSnapshot<Error = DeviceSnapshotError>> {
        None
    }

    fn support_load_snapshot(
        &mut self,
    ) -> Option<&mut dyn LoadSnapshot<Error = DeviceSnapshotError>> {
        None
    }
}
