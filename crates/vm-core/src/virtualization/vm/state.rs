use serde::Deserialize;
use serde::Serialize;

use crate::virtualization::vm::error::VmError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmState {
    Created,
    Running,
    Paused,
}

impl VmState {
    fn ensure_is(&self, state: VmState) -> Result<(), VmError> {
        if *self != state {
            return Err(VmError::VmState { current: *self });
        }

        Ok(())
    }

    fn ensure_is_not(&self, state: VmState) -> Result<(), VmError> {
        if *self == state {
            return Err(VmError::VmState { current: *self });
        }

        Ok(())
    }

    pub fn ensure_is_created(&self) -> Result<(), VmError> {
        self.ensure_is(VmState::Created)
    }

    pub fn ensure_is_running(&self) -> Result<(), VmError> {
        self.ensure_is(VmState::Running)
    }

    pub fn ensure_is_paused(&self) -> Result<(), VmError> {
        self.ensure_is(VmState::Paused)
    }

    pub fn ensure_is_not_running(&self) -> Result<(), VmError> {
        self.ensure_is_not(VmState::Running)
    }
}
