use std::sync::Arc;

use applevisor::gic::GicConfig;
use applevisor::memory::MemPerms;
use applevisor::prelude::HypervisorError;
use applevisor_sys::hv_error_t;
use applevisor_sys::hv_gic_config_create;
use applevisor_sys::hv_gic_config_set_distributor_base;
use applevisor_sys::hv_gic_config_set_msi_interrupt_range;
use applevisor_sys::hv_gic_config_set_msi_region_base;
use applevisor_sys::hv_gic_config_set_redistributor_base;
use applevisor_sys::hv_gic_config_t;
use applevisor_sys::hv_gic_create;
use applevisor_sys::hv_vm_config_create;
use applevisor_sys::hv_vm_config_set_el2_enabled;
use applevisor_sys::hv_vm_create;
use applevisor_sys::hv_vm_map;

use crate::arch::aarch64::layout::GIC_DISTRIBUTOR;
use crate::arch::aarch64::layout::GIC_MSI;
use crate::arch::aarch64::layout::GIC_REDISTRIBUTOR;
use crate::arch::aarch64::layout::RAM_BASE;
use crate::arch::irq::InterruptController;
use crate::error::Error;
use crate::error::Result;
use crate::virt::Virt;
use crate::virt::Vm;
use crate::virt::hvp::irq_chip::HvpGicV3;
use crate::virt::hvp::vcpu::HvpVcpu;
use crate::virt::vcpu::Vcpu;
use crate::virt::vm::SetUserMemoryRegionFlags;

pub(crate) mod vcpu;

mod irq_chip;

macro_rules! hv_unsafe_call {
    ($x:expr) => {{
        let ret = unsafe { $x };
        match ret {
            x if x == hv_error_t::HV_SUCCESS as i32 => Ok(()),
            code => Err(HypervisorError::from(code)),
        }
    }};
}

pub(crate) use hv_unsafe_call;

impl From<SetUserMemoryRegionFlags> for MemPerms {
    fn from(flags: SetUserMemoryRegionFlags) -> Self {
        match flags {
            SetUserMemoryRegionFlags::ReadWriteExec => MemPerms::ReadWriteExec,
        }
    }
}

pub struct AppleHypervisorVm {}

impl Vm for AppleHypervisorVm {
    fn create_vcpu(&self, vcpu_id: usize) -> Result<Box<dyn Vcpu>> {
        let vcpu = HvpVcpu::new(vcpu_id);

        Ok(Box::new(vcpu))
    }

    fn create_irq_chip(&self) -> Result<Arc<dyn InterruptController>> {
        let distributor_base = GIC_DISTRIBUTOR;
        let redistributor_base = GIC_REDISTRIBUTOR;
        let msi_base = GIC_MSI;
        let ram_base = RAM_BASE;

        let gic_config: hv_gic_config_t = unsafe { hv_gic_config_create() };

        let distributor_base_alignment = GicConfig::get_distributor_base_alignment()
            .map_err(|err| Error::InterruptControllerFailed(err.to_string()))?;
        let redistributor_base_alignment = GicConfig::get_redistributor_base_alignment()
            .map_err(|err| Error::InterruptControllerFailed(err.to_string()))?;
        let msi_region_base_alignment = GicConfig::get_msi_region_base_alignment()
            .map_err(|err| Error::InterruptControllerFailed(err.to_string()))?;

        let gic_distributor_size = GicConfig::get_distributor_size().map_err(|err| {
            Error::InterruptControllerFailed(format!(
                "Failed to get the size of distributor, {:?}",
                err
            ))
        })?;

        let gic_redistributor_region_size =
            GicConfig::get_redistributor_region_size().map_err(|err| {
                Error::InterruptControllerFailed(format!(
                    "Failed to get the size of redistributor region, {:?}",
                    err
                ))
            })?;

        let gic_msi_region_size = GicConfig::get_msi_region_size().map_err(|err| {
            Error::InterruptControllerFailed(format!(
                "Failed to get the size of msi region, {:?}",
                err
            ))
        })?;

        {
            // Setup distributor
            if !distributor_base.is_multiple_of(distributor_base_alignment as u64) {
                return Err(Error::InterruptControllerFailed(
                    "The base address of gic distributor is not aligned".to_string(),
                ));
            }
            hv_unsafe_call!(hv_gic_config_set_distributor_base(
                gic_config,
                distributor_base
            ))?;
        }

        {
            // Setup redistributor
            if !redistributor_base.is_multiple_of(redistributor_base_alignment as u64) {
                return Err(Error::InterruptControllerFailed(
                    "The base address of gic redistributor is not aligned".to_string(),
                ));
            }
            if distributor_base + gic_distributor_size as u64 > redistributor_base {
                return Err(Error::InterruptControllerFailed(
                    "distributor too large".to_string(),
                ));
            }
            hv_unsafe_call!(hv_gic_config_set_redistributor_base(
                gic_config,
                redistributor_base
            ))?;
        }

        {
            // Setup msi
            if !msi_base.is_multiple_of(msi_region_base_alignment as u64) {
                return Err(Error::InterruptControllerFailed(
                    "The base address of gic msi is not aligned".to_string(),
                ));
            }
            if redistributor_base + gic_redistributor_region_size as u64 > msi_base {
                return Err(Error::InterruptControllerFailed(
                    "redistributor too large".to_string(),
                ));
            }
            hv_unsafe_call!(hv_gic_config_set_msi_region_base(gic_config, msi_base))?;
            hv_unsafe_call!(hv_gic_config_set_msi_interrupt_range(gic_config, 256, 256))?;
        }

        if msi_base + gic_msi_region_size as u64 > ram_base {
            return Err(Error::InterruptControllerFailed(
                "msi region too large".to_string(),
            ));
        }

        hv_unsafe_call!(hv_gic_create(gic_config))?;

        Ok(Arc::new(HvpGicV3::new(
            distributor_base,
            redistributor_base,
            msi_base,
        )))
    }

    fn set_user_memory_region(
        &self,
        userspace_addr: u64,
        guest_phys_addr: u64,
        memory_size: usize,
        flags: SetUserMemoryRegionFlags,
    ) -> Result<()> {
        hv_unsafe_call!(hv_vm_map(
            userspace_addr as _,
            guest_phys_addr,
            memory_size,
            MemPerms::from(flags) as u64,
        ))?;

        Ok(())
    }
}

#[derive(Default)]
pub struct AppleHypervisor;

impl Virt for AppleHypervisor {
    fn create_vm(&self) -> Result<Arc<dyn Vm>> {
        let vm_config = unsafe { hv_vm_config_create() };
        hv_unsafe_call!(hv_vm_config_set_el2_enabled(vm_config, true))?;
        hv_unsafe_call!(hv_vm_create(vm_config))?;

        Ok(Arc::new(AppleHypervisorVm {}))
    }
}
