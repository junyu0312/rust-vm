use std::cell::OnceCell;
use std::sync::Arc;

use anyhow::anyhow;
use applevisor::gic::GicConfig;
use applevisor::memory::MemPerms;
use applevisor::memory::Memory;
use applevisor::vm::GicEnabled;
use applevisor::vm::VirtualMachine;
use applevisor::vm::VirtualMachineConfig;
use applevisor::vm::VirtualMachineInstance;
use applevisor_sys::hv_sys_reg_t;

use crate::arch::aarch64::AArch64;
use crate::arch::aarch64::layout::GIC_DISTRIBUTOR;
use crate::arch::aarch64::layout::GIC_REDISTRIBUTOR;
use crate::arch::vm_exit::aarch64::HandleVmExitResult;
use crate::arch::vm_exit::aarch64::handle_vm_exit;
use crate::device::IoAddressSpace;
use crate::device::mmio::MmioLayout;
use crate::mm::manager::MemoryAddressSpace;
use crate::vcpu::Vcpu;
use crate::vcpu::arch::aarch64::AArch64Vcpu;
use crate::vcpu::arch::aarch64::reg::CoreRegister;
use crate::virt::Virt;
use crate::virt::VirtError;
use crate::virt::hvp::irq_chip::HvpGicV3;
use crate::virt::hvp::mm::HvpAllocator;
use crate::virt::hvp::vcpu::HvpVcpu;

pub(crate) mod vcpu;

mod irq_chip;
mod mm;

pub struct Hvp {
    vm: Arc<VirtualMachineInstance<GicEnabled>>,
    gic_chip: Arc<HvpGicV3>,
    vcpus: OnceCell<Vec<HvpVcpu>>,
}

impl Virt for Hvp {
    type Arch = AArch64;

    type Vcpu = HvpVcpu;

    type Memory = Memory;

    type Irq = HvpGicV3;

    fn new() -> Result<Self, VirtError> {
        let mut vm_config = VirtualMachineConfig::default();
        vm_config
            .set_el2_enabled(true)
            .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;

        let mut gic_config = GicConfig::default();
        let gic_distributor_size = GicConfig::get_distributor_size().map_err(|err| {
            VirtError::InterruptControllerFailed(format!(
                "Failed to get the size of distributor, {:?}",
                err
            ))
        })?;
        let distributor_base_alignment = GicConfig::get_distributor_base_alignment()
            .map_err(|err| VirtError::InterruptControllerFailed(err.to_string()))?;
        let redistributor_base_alignment = GicConfig::get_redistributor_base_alignment()
            .map_err(|err| VirtError::InterruptControllerFailed(err.to_string()))?;

        {
            // Setup distributor
            if !GIC_DISTRIBUTOR.is_multiple_of(distributor_base_alignment as u64) {
                return Err(VirtError::InterruptControllerFailed(
                    "The base address of gic distributor is not aligned".to_string(),
                ));
            }
            gic_config
                .set_distributor_base(GIC_DISTRIBUTOR)
                .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;
        }

        {
            // Setup redistributor
            if !GIC_REDISTRIBUTOR.is_multiple_of(redistributor_base_alignment as u64) {
                return Err(VirtError::InterruptControllerFailed(
                    "The base address of gic redistributor is not aligned".to_string(),
                ));
            }
            if GIC_DISTRIBUTOR + gic_distributor_size as u64 > GIC_REDISTRIBUTOR {
                return Err(VirtError::InterruptControllerFailed(
                    "distributor too large".to_string(),
                ));
            }
            gic_config
                .set_redistributor_base(GIC_REDISTRIBUTOR)
                .map_err(|err| VirtError::FailedInitialize(err.to_string()))?;
        }

        let vm = VirtualMachine::with_gic(vm_config, gic_config).map_err(|err| {
            VirtError::FailedInitialize(
                format!("hvp: Failed to create a vm instance, reason: {}", err).to_string(),
            )
        })?;
        let vm = Arc::new(vm);

        let gic_chip = HvpGicV3::new(GIC_DISTRIBUTOR, GIC_REDISTRIBUTOR, vm.clone());
        let gic_chip = Arc::new(gic_chip);

        Ok(Hvp {
            vm,
            gic_chip,
            vcpus: OnceCell::default(),
        })
    }

    fn init_irq(&mut self) -> anyhow::Result<Arc<Self::Irq>> {
        Ok(self.gic_chip.clone())
    }

    fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()> {
        let mut vcpus = Vec::with_capacity(num_vcpus);

        for vcpu_id in 0..num_vcpus {
            let vcpu = self.vm.vcpu_create()?;
            vcpu.set_sys_reg(hv_sys_reg_t::MPIDR_EL1, vcpu_id as u64)?;
            vcpus.push(HvpVcpu::new(vcpu_id as u64, vcpu));
        }

        self.vcpus
            .set(vcpus)
            .map_err(|_| anyhow!("vcpu is ready initialized"))?;

        Ok(())
    }

    fn init_memory(
        &mut self,
        _mmio_layout: &MmioLayout,
        memory: &mut MemoryAddressSpace<Memory>,
    ) -> anyhow::Result<()> {
        let allocator = HvpAllocator {
            vm: self.vm.as_ref(),
        };

        for region in memory {
            region.alloc(&allocator)?;

            let memory = region.memory.get_mut().unwrap();
            memory.map(region.gpa, MemPerms::ReadWriteExec)?;
        }

        Ok(())
    }

    fn post_init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_vcpu_mut(&mut self, vcpu_id: u64) -> anyhow::Result<Option<&mut HvpVcpu>> {
        let vcpu = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?
            .get_mut(vcpu_id as usize);

        Ok(vcpu)
    }

    fn get_vcpus(&self) -> anyhow::Result<&Vec<Self::Vcpu>> {
        let vcpus = self
            .vcpus
            .get()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?;

        Ok(vcpus)
    }

    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<Self::Vcpu>> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?;

        Ok(vcpus)
    }

    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()> {
        // TODO: support smp, fork for per vcpu
        {
            loop {
                let vcpu = self.get_vcpu_mut(0)?.unwrap();
                let vm_exit_info = vcpu.run(device.mmio_layout())?;

                let r = handle_vm_exit(vcpu, vm_exit_info, device)?;

                match r {
                    HandleVmExitResult::Continue => (),
                    HandleVmExitResult::NextInstruction => {
                        let pc = vcpu.get_core_reg(CoreRegister::PC)?;
                        vcpu.set_core_reg(CoreRegister::PC, pc + 4)?;
                    }
                }
            }
        }
    }
}
