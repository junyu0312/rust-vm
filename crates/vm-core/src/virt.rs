use std::sync::Arc;

use crate::error::Error;
use crate::virt::vm::HypervisorVm;

#[cfg(feature = "kvm")]
#[allow(dead_code)]
pub mod kvm;

#[cfg(feature = "hvp")]
pub mod hvp;

pub mod vcpu;
pub mod vm;

pub trait Virt {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, Error>;
}
