use std::cell::OnceCell;

use anyhow::anyhow;
use applevisor::memory::MemPerms;
use applevisor::memory::Memory;
use applevisor::vm::GicDisabled;
use applevisor::vm::VirtualMachine;
use applevisor::vm::VirtualMachineInstance;

use crate::device::pio::IoAddressSpace;
use crate::mm::manager::MemoryAddressSpace;
use crate::virt::Virt;
use crate::virt::VirtError;
use crate::virt::hvp::irq_chip::HvpGicV3;
use crate::virt::hvp::mm::HvpAllocator;
use crate::virt::hvp::vcpu::HvpVcpu;

mod irq_chip;
mod mm;
mod vcpu;

pub struct Hvp {
    vm: VirtualMachineInstance<GicDisabled>,
    vcpus: OnceCell<Vec<HvpVcpu>>,
}

impl Virt for Hvp {
    type Vcpu = HvpVcpu;

    type Memory = Memory;

    type Irq = HvpGicV3;

    fn new() -> Result<Self, VirtError> {
        let vm = VirtualMachine::new().map_err(|_| {
            VirtError::FailedInitialize("hvp: Failed to create a vm instance".to_string())
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

    fn init_memory(&mut self, memory: &mut MemoryAddressSpace<Memory>) -> anyhow::Result<()> {
        let allocator = HvpAllocator { vm: &self.vm };

        for region in memory {
            region.alloc(&allocator)?;

            let memory = region.memory.get_mut().unwrap();
            memory.map(region.gpa, MemPerms::ReadWriteExec)?;
        }

        Ok(())
    }

    fn post_init(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn get_vcpu_mut(&mut self, vcpu_id: u64) -> anyhow::Result<Option<&mut HvpVcpu>> {
        let vcpu = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpu is not initialized"))?
            .get_mut(vcpu_id as usize);

        Ok(vcpu)
    }

    fn run(&mut self, _device: &mut IoAddressSpace) -> anyhow::Result<()> {
        todo!()
    }
}
