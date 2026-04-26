use std::sync::Arc;

use applevisor::gic::GicConfig;
use applevisor::memory::MemPerms;
use applevisor_sys::hv_error_t;
use applevisor_sys::hv_gic_config_create;
use applevisor_sys::hv_gic_config_set_distributor_base;
use applevisor_sys::hv_gic_config_set_msi_interrupt_range;
use applevisor_sys::hv_gic_config_set_msi_region_base;
use applevisor_sys::hv_gic_config_set_redistributor_base;
use applevisor_sys::hv_gic_config_t;
use applevisor_sys::hv_gic_create;
use applevisor_sys::hv_vm_map;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::aarch64::layout::GIC_DISTRIBUTOR;
use crate::arch::aarch64::layout::GIC_MSI;
use crate::arch::aarch64::layout::GIC_REDISTRIBUTOR;
use crate::arch::aarch64::layout::RAM_BASE;
use crate::arch::irq::InterruptController;
use crate::cpu::vm_exit::VmExit;
use crate::virtualization::hvp::hv_unsafe_call;
use crate::virtualization::hvp::irq_chip::HvpGicV3;
use crate::virtualization::hvp::vcpu::HvpVcpu;
use crate::virtualization::vcpu::HypervisorVcpu;
use crate::virtualization::vm::HypervisorVm;
use crate::virtualization::vm::SetUserMemoryRegionFlags;
use crate::virtualization::vm::error::VmError;

impl From<SetUserMemoryRegionFlags> for MemPerms {
    fn from(flags: SetUserMemoryRegionFlags) -> Self {
        match flags {
            SetUserMemoryRegionFlags::ReadWriteExec => MemPerms::ReadWriteExec,
        }
    }
}

#[derive(Default)]
pub struct AppleHypervisorVm {}

impl HypervisorVm for AppleHypervisorVm {
    fn create_vcpu(
        &self,
        vcpu_id: usize,
        mm: Arc<MemoryAddressSpace>,
        vm_exit_handler: Arc<dyn VmExit>,
    ) -> Result<Box<dyn HypervisorVcpu>, VmError> {
        let vcpu = Box::new(
            HvpVcpu::new(vcpu_id, mm, vm_exit_handler)
                .map_err(|err| VmError::CreateVcpuError(Box::new(err)))?,
        );

        Ok(vcpu as _)
    }

    fn create_irq_chip(&self) -> Result<Arc<dyn InterruptController>, VmError> {
        let distributor_base = GIC_DISTRIBUTOR;
        let redistributor_base = GIC_REDISTRIBUTOR;
        let msi_base = GIC_MSI;
        let ram_base = RAM_BASE;

        let gic_config: hv_gic_config_t = unsafe { hv_gic_config_create() };

        let distributor_base_alignment = GicConfig::get_distributor_base_alignment()?;
        let redistributor_base_alignment = GicConfig::get_redistributor_base_alignment()?;
        let msi_region_base_alignment = GicConfig::get_msi_region_base_alignment()?;

        let gic_distributor_size = GicConfig::get_distributor_size()?;
        let gic_redistributor_region_size = GicConfig::get_redistributor_region_size()?;
        let gic_msi_region_size = GicConfig::get_msi_region_size()?;

        {
            // Setup distributor
            if !distributor_base.is_multiple_of(distributor_base_alignment as u64) {
                return Err(VmError::CreateIrqChipError(
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
                return Err(VmError::CreateIrqChipError(
                    "The base address of gic redistributor is not aligned".to_string(),
                ));
            }
            if distributor_base + gic_distributor_size as u64 > redistributor_base {
                return Err(VmError::CreateIrqChipError(
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
                return Err(VmError::CreateIrqChipError(
                    "The base address of gic msi is not aligned".to_string(),
                ));
            }
            if redistributor_base + gic_redistributor_region_size as u64 > msi_base {
                return Err(VmError::CreateIrqChipError(
                    "redistributor too large".to_string(),
                ));
            }
            hv_unsafe_call!(hv_gic_config_set_msi_region_base(gic_config, msi_base))?;
            hv_unsafe_call!(hv_gic_config_set_msi_interrupt_range(gic_config, 256, 256))?;
        }

        if msi_base + gic_msi_region_size as u64 > ram_base {
            return Err(VmError::CreateIrqChipError(
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
    ) -> Result<(), VmError> {
        hv_unsafe_call!(hv_vm_map(
            userspace_addr as _,
            guest_phys_addr,
            memory_size,
            MemPerms::from(flags) as u64,
        ))
        .map_err(|err| VmError::SetUserMemoryRegionError(err.to_string()))?;

        Ok(())
    }
}
