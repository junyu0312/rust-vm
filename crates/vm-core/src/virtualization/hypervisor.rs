use std::sync::Arc;

use crate::virtualization::hypervisor::error::HypervisorError;
use crate::virtualization::vm::HypervisorVm;

pub mod error;

pub trait Hypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError>;
}
