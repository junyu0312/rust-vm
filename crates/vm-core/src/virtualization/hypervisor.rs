use std::sync::Arc;

use thiserror::Error;

use crate::virtualization::vm::HypervisorVm;

#[derive(Error, Debug)]
pub enum HypervisorError {
    #[error("Failed to create vm: {0}")]
    CreateVm(String),
}

pub trait Hypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError>;
}
