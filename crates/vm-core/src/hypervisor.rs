use std::sync::Arc;

use crate::hypervisor::vm::HypervisorVm;

#[cfg(feature = "kvm")]
#[allow(dead_code)]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub mod vcpu;
pub mod vm;

#[derive(Debug, thiserror::Error)]
pub enum HypervisorError {
    #[cfg(feature = "hvp")]
    #[error("{0}")]
    ApplevisorError(#[from] applevisor::error::HypervisorError),
}

pub trait Hypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError>;
}
