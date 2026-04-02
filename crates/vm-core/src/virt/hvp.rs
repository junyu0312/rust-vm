use std::sync::Arc;

use applevisor::prelude::HypervisorError;
use applevisor_sys::hv_error_t;
use applevisor_sys::hv_vm_config_create;
use applevisor_sys::hv_vm_config_set_el2_enabled;
use applevisor_sys::hv_vm_create;

use crate::error::Result;
use crate::virt::HypervisorVm;
use crate::virt::Virt;
use crate::virt::hvp::vm::AppleHypervisorVm;

mod irq_chip;
mod vcpu;
mod vm;

macro_rules! hv_unsafe_call {
    ($x:expr) => {{
        let ret = unsafe { $x };
        match ret {
            x if x == hv_error_t::HV_SUCCESS as i32 => Ok(()),
            code => Err(HypervisorError::from(code)),
        }
    }};
}

use hv_unsafe_call;

#[derive(Default)]
pub struct AppleHypervisor;

impl Virt for AppleHypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>> {
        let vm_config = unsafe { hv_vm_config_create() };
        hv_unsafe_call!(hv_vm_config_set_el2_enabled(vm_config, true))?;
        hv_unsafe_call!(hv_vm_create(vm_config))?;

        Ok(Arc::new(AppleHypervisorVm))
    }
}
