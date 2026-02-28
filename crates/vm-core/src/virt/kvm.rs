use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use kvm_bindings::*;
use kvm_ioctls::*;
use memmap2::MmapMut;
use vm_mm::allocator::mmap_allocator::MmapAllocator;
use vm_mm::manager::MemoryAddressSpace;

use crate::arch::Arch;
use crate::arch::irq::InterruptController;
use crate::device::mmio::MmioLayout;
use crate::device::vm_exit::DeviceVmExitHandler;
use crate::error::Error;
use crate::virt::Vcpu;
use crate::virt::Virt;
use crate::virt::kvm::irq_chip::KvmIRQ;
use crate::virt::kvm::vcpu::KvmVcpu;

mod arch;
mod irq_chip;
mod vcpu;

pub trait KvmArch {
    fn arch_post_init(&mut self) -> anyhow::Result<()>;
}

#[allow(unused)]
pub struct KvmVirt<A: Arch> {
    kvm: Kvm,
    vm_fd: Arc<VmFd>,
    vcpus: OnceCell<Vec<KvmVcpu>>,
    _mark: PhantomData<A>,
}

impl<A> Virt for KvmVirt<A>
where
    A: Arch,
    KvmVcpu: Vcpu<A>,
    Self: KvmArch,
{
    type Arch = A;
    type Vcpu = KvmVcpu;
    type Memory = MmapMut;

    fn new(_cpu_number: usize) -> Result<Self, Error> {
        let kvm = Kvm::new()
            .map_err(|_| Error::FailedInitialize("kvm: Failed to open /dev/kvm".to_string()))?;

        let vm_fd = Arc::new(
            kvm.create_vm()
                .map_err(|_| Error::FailedInitialize("kvm: Failed to create_vm".to_string()))?,
        );

        Ok(KvmVirt {
            kvm,
            vm_fd,
            vcpus: OnceCell::new(),
            _mark: PhantomData,
        })
    }

    fn init_irq(&mut self) -> anyhow::Result<Arc<dyn InterruptController>> {
        Ok(Arc::new(KvmIRQ::new(self.vm_fd.clone())?))
    }

    fn init_memory(
        &mut self,
        _mmio_layout: &MmioLayout,
        memory: &mut MemoryAddressSpace<MmapMut>,
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

    fn get_vcpu_number(&self) -> usize {
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
        self.vcpus.get().ok_or_else(|| anyhow!("vcpus is not init"))
    }

    fn get_vcpus_mut(&mut self) -> anyhow::Result<&mut Vec<KvmVcpu>> {
        self.vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))
    }

    fn run(&mut self, device: Arc<Mutex<dyn DeviceVmExitHandler>>) -> anyhow::Result<()> {
        let vcpus = self
            .vcpus
            .get_mut()
            .ok_or_else(|| anyhow!("vcpus is not init"))?;

        assert_eq!(vcpus.len(), 1);

        let mmio_layout = device.lock().unwrap().mmio_layout();

        vcpus.get_mut(0).unwrap().run(mmio_layout.as_ref())?;

        Ok(())
    }
}
