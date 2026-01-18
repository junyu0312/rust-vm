use std::cell::OnceCell;
use std::sync::Arc;

use anyhow::anyhow;
use kvm_bindings::*;
use kvm_ioctls::*;

use crate::device::pio::IoAddressSpace;
use crate::mm::manager::MemoryRegions;
use crate::vcpu::Vcpu;
use crate::virt::Virt;
use crate::virt::kvm::irq_chip::KvmIRQ;
use crate::virt::kvm::vcpu::KvmVcpu;

mod arch;
mod irq_chip;
mod vcpu;

pub struct KvmVirt {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
}

impl KvmVirt {
    fn create_vcpu(&self, vcpu_id: u64) -> anyhow::Result<KvmVcpu> {
        let vcpu_fd = self.vm_fd.create_vcpu(vcpu_id)?;

        Ok(KvmVcpu { vcpu_id, vcpu_fd })
    }
}

impl Virt for KvmVirt {
    type Vcpu = KvmVcpu;
    type Irq = KvmIRQ;

    fn new() -> anyhow::Result<Self> {
        let kvm = Kvm::new()?;

        let vm_fd = Arc::new(kvm.create_vm()?);

        Ok(KvmVirt {
            kvm,
            vm_fd,
            vcpus: OnceCell::new(),
        })
    }

    fn init_irq(&mut self) -> anyhow::Result<KvmIRQ> {
        KvmIRQ::new(self.vm_fd.clone())
    }

    fn init_vcpus(&mut self, num_vcpus: usize) -> anyhow::Result<()> {
        let mut vcpus = Vec::with_capacity(num_vcpus);

        for vcpu_id in 0..num_vcpus {
            let vcpu = self.create_vcpu(vcpu_id as u64)?;

            vcpu.init_arch_vcpu(&self.kvm)?;

            vcpus.push(vcpu);
        }

        self.vcpus
            .set(vcpus)
            .map_err(|_| anyhow!("vcpus is already init"))?;

        Ok(())
    }

    fn init_memory(&mut self, memory: &MemoryRegions) -> anyhow::Result<()> {
        for (slot, region) in memory.into_iter().enumerate() {
            unsafe {
                self.vm_fd
                    .set_user_memory_region(kvm_userspace_memory_region {
                        slot: slot as u32,
                        flags: 0,
                        guest_phys_addr: region.gpa as u64,
                        memory_size: region.len as u64,
                        userspace_addr: region.as_u64(),
                    })?;
            }
        }

        Ok(())
    }

    fn post_init(&mut self) -> anyhow::Result<()> {
        self.arch_post_init()?;

        Ok(())
    }

    fn get_vcpu_mut(&mut self, vcpu: u64) -> anyhow::Result<Option<&mut KvmVcpu>> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))?;

        Ok(vcpus.get_mut(vcpu as usize))
    }

    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))?;

        assert_eq!(vcpus.len(), 1);

        vcpus.get_mut(0).unwrap().run(device)?;

        Ok(())
    }
}
