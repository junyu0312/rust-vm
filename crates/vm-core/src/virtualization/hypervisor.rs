use std::sync::Arc;

use thiserror::Error;

use crate::virtualization::vm::HypervisorVm;

#[derive(Debug, Error)]
pub enum HypervisorError {
    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),
}

pub trait Hypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError>;
}
