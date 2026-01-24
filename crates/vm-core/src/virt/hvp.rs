use std::cell::OnceCell;

use anyhow::anyhow;
use applevisor::memory::MemPerms;
use applevisor::memory::Memory;
use applevisor::vm::GicDisabled;
use applevisor::vm::VirtualMachine;
use applevisor::vm::VirtualMachineInstance;

use crate::arch::aarch64::AArch64;
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
use crate::virt::vm_exit::HandleVmExitResult;
use crate::virt::vm_exit::handle_vm_exit;

pub(crate) mod vcpu;

mod irq_chip;
mod mm;

pub struct Hvp {
    vm: VirtualMachineInstance<GicDisabled>,
    vcpus: OnceCell<Vec<HvpVcpu>>,
}

impl Virt for Hvp {
    type Arch = AArch64;

    type Vcpu = HvpVcpu;

    type Memory = Memory;

    type Irq = HvpGicV3;

    fn new() -> Result<Self, VirtError> {
        let vm = VirtualMachine::new().map_err(|err| {
            VirtError::FailedInitialize(
                format!("hvp: Failed to create a vm instance, reason: {}", err).to_string(),
            )
        })?;

        Ok(Hvp {
            vm,
            vcpus: OnceCell::default(),
        })
    }

    fn init_irq(&mut self) -> anyhow::Result<Self::Irq> {
        Ok(HvpGicV3)
    }

    fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()> {
        let mut vcpus = Vec::with_capacity(num_vcpus);

        for vcpu_id in 0..num_vcpus {
            let vcpu = self.vm.vcpu_create()?;
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
        let allocator = HvpAllocator { vm: &self.vm };

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
                    HandleVmExitResult::Continue => {
                        let pc = vcpu.get_core_reg(CoreRegister::PC)?;
                        vcpu.set_core_reg(CoreRegister::PC, pc + 4)?;
                    }
                }
            }
        }
    }
}
