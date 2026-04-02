use std::sync::Arc;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_vm_config_create;
use applevisor_sys::hv_vm_config_set_el2_enabled;
use applevisor_sys::hv_vm_create;

use crate::hypervisor::Hypervisor;
use crate::hypervisor::HypervisorError;
use crate::hypervisor::HypervisorVm;
use crate::hypervisor::hvp::vm::AppleHypervisorVm;

mod irq_chip;
mod vcpu;
mod vm;

macro_rules! hv_unsafe_call {
    ($x:expr) => {{
        let ret = unsafe { $x };
        match ret {
            x if x == hv_error_t::HV_SUCCESS as i32 => Ok(()),
            code => Err(applevisor::prelude::HypervisorError::from(code)),
        }
    }};
}

use hv_unsafe_call;

#[derive(Default)]
pub struct AppleHypervisor;

impl Hypervisor for AppleHypervisor {
    fn create_vm(&self) -> Result<Arc<dyn HypervisorVm>, HypervisorError> {
        let vm_config = unsafe { hv_vm_config_create() };
        hv_unsafe_call!(hv_vm_config_set_el2_enabled(vm_config, true))?;
        hv_unsafe_call!(hv_vm_create(vm_config))?;

        Ok(Arc::new(AppleHypervisorVm))
    }
}
