use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::anyhow;
use kvm_bindings::*;
use kvm_ioctls::*;

use crate::arch::Arch;
use crate::device::IoAddressSpace;
use crate::device::mmio::MmioLayout;
use crate::mm::allocator::mmap_allocator::MmapAllocator;
use crate::mm::allocator::mmap_allocator::MmapMemory;
use crate::mm::manager::MemoryAddressSpace;
use crate::vcpu::Vcpu;
use crate::virt::Virt;
use crate::virt::error::VirtError;
use crate::virt::kvm::irq_chip::KvmIRQ;
use crate::virt::kvm::vcpu::KvmVcpu;

mod arch;
mod irq_chip;
mod vcpu;

pub trait KvmArch {
    fn arch_post_init(&mut self) -> anyhow::Result<()>;
}

pub struct KvmVirt<A: Arch> {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
    _mark: PhantomData<A>,
}

impl<A> KvmVirt<A>
where
    A: Arch,
{
    fn create_vcpu(&self, vcpu_id: u64) -> anyhow::Result<KvmVcpu> {
        let vcpu_fd = self.vm_fd.create_vcpu(vcpu_id)?;

        Ok(KvmVcpu { vcpu_id, vcpu_fd })
    }
}

impl<A> Virt for KvmVirt<A>
where
    A: Arch,
    KvmVcpu: Vcpu<A>,
    Self: KvmArch,
{
    type Arch = A;
    type Vcpu = KvmVcpu;
    type Memory = MmapMemory;
    type Irq = KvmIRQ;

    fn new() -> Result<Self, VirtError> {
        let kvm = Kvm::new()
            .map_err(|_| VirtError::FailedInitialize("kvm: Failed to open /dev/kvm".to_string()))?;

        let vm_fd =
            Arc::new(kvm.create_vm().map_err(|_| {
                VirtError::FailedInitialize("kvm: Failed to create_vm".to_string())
            })?);

        Ok(KvmVirt {
            kvm,
            vm_fd,
            vcpus: OnceCell::new(),
            _mark: PhantomData,
        })
    }

    fn init_irq(&mut self) -> anyhow::Result<Arc<KvmIRQ>> {
        Ok(Arc::new(KvmIRQ::new(self.vm_fd.clone())?))
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

    fn init_memory(
        &mut self,
        _mmio_layout: &MmioLayout,
        memory: &mut MemoryAddressSpace<MmapMemory>,
        _memory_size: u64,
    ) -> anyhow::Result<()> {
        let allocator = MmapAllocator;

        for (slot, region) in memory.into_iter().enumerate() {
            region.alloc(&allocator)?;

            unsafe {
                self.vm_fd
                    .set_user_memory_region(kvm_userspace_memory_region {
                        slot: slot as u32,
                        flags: 0,
                        guest_phys_addr: region.gpa,
                        memory_size: region.len as u64,
                        userspace_addr: region.to_hva().unwrap() as u64,
                    })?;
            }
        }

        Ok(())
    }

    fn post_init(&mut self) -> anyhow::Result<()> {
        self.arch_post_init()?;

        Ok(())
    }

    fn get_layout(&self) -> &<Self::Arch as Arch>::Layout {
        todo!()
    }

    fn get_layout_mut(&mut self) -> &mut <Self::Arch as Arch>::Layout {
        todo!()
    }

    fn get_vcpu_mut(&mut self, vcpu: u64) -> anyhow::Result<Option<&mut KvmVcpu>> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))?;

        Ok(vcpus.get_mut(vcpu as usize))
    }

    fn get_vcpus(&self) -> anyhow::Result<&Vec<KvmVcpu>> {
        Ok(self
            .vcpus
            .get()
            .ok_or_else(|| anyhow!("vcpus is not init"))?)
    }

    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<KvmVcpu>> {
        Ok(self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))?)
    }

    fn run(&mut self, device: &mut IoAddressSpace) -> anyhow::Result<()> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))?;

        assert_eq!(vcpus.len(), 1);

        vcpus.get_mut(0).unwrap().run(device.mmio_layout())?;

        Ok(())
    }
}
