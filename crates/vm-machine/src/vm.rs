use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use kvm_bindings::kvm_pit_config;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::Kvm;
use kvm_ioctls::VcpuExit;
use kvm_ioctls::VmFd;
use tracing::error;
use vm_bootloader::BootLoader;
use vm_bootloader::linux::bzimage::BzImage;
use vm_core::mm::manager::MemoryRegions;
use vm_core::mm::region::MemoryRegion;
use vm_core::virt::kvm::vcpu::KvmVcpu;
use vm_device::bus::io_address_space::IoAddressSpace;

use crate::device::init_device;
use crate::firmware::bios::Bios;
use crate::kvm::irq::KvmIRQ;

pub struct VmBuilder {
    pub memory_size: usize,
    pub vcpus: usize,
    pub kernel: PathBuf,
    pub initramfs: Option<PathBuf>,
    pub cmdline: Option<String>,
}

#[allow(dead_code)]
pub struct Vm {
    memory: MemoryRegions,
    memory_size: usize,

    vcpus: Vec<KvmVcpu>,

    devices: IoAddressSpace,
}

impl VmBuilder {
    fn init_mm(&self) -> anyhow::Result<MemoryRegions> {
        let memory_region = MemoryRegion::new(0, self.memory_size)?;

        let mut memory_regions = MemoryRegions::default();
        memory_regions
            .try_insert(memory_region)
            .map_err(|_| anyhow!("Failed to insert memory_region"))?;

        Ok(memory_regions)
    }

    fn create_vcpu(&self, vm_fd: &VmFd, id: u64) -> anyhow::Result<KvmVcpu> {
        let vcpu_fd = vm_fd.create_vcpu(id)?;

        Ok(KvmVcpu {
            vcpu_id: id,
            vcpu_fd,
        })
    }

    fn init_vcpus(&self, kvm: &Kvm, vm_fd: &VmFd) -> anyhow::Result<Vec<KvmVcpu>> {
        let mut vcpus = Vec::with_capacity(self.vcpus);

        for vcpu_id in 0..self.vcpus {
            let vcpu_id = vcpu_id as u64;
            let vcpu_fd = self.create_vcpu(vm_fd, vcpu_id)?;

            vcpu_fd.init_arch_vcpu(kvm)?;

            vcpus.push(vcpu_fd);
        }

        Ok(vcpus)
    }

    fn arch_init(&self, memory: &mut MemoryRegions, vm_fd: &VmFd) -> anyhow::Result<()> {
        {
            let bios = Bios;
            bios.init(memory, self.memory_size)?;
        }

        {
            let pit_config = kvm_pit_config::default();
            vm_fd.create_pit2(pit_config).unwrap();
        }

        Ok(())
    }

    pub fn build(&self) -> anyhow::Result<Vm> {
        let kvm = Kvm::new()?;

        let vm_fd = Arc::new(kvm.create_vm()?);

        let kvm_irq = Arc::new(KvmIRQ::new(vm_fd.clone())?);

        let mut vcpus = self.init_vcpus(&kvm, &vm_fd)?;

        let mut memory = self.init_mm()?;
        for (slot, region) in memory.into_iter().enumerate() {
            unsafe {
                vm_fd.set_user_memory_region(kvm_userspace_memory_region {
                    slot: slot as u32,
                    flags: 0,
                    guest_phys_addr: region.gpa as u64,
                    memory_size: region.len as u64,
                    userspace_addr: region.as_u64(),
                })?;
            }
        }

        let devices = init_device(kvm_irq)?;

        let bz_image = BzImage::new(
            &self.kernel,
            self.initramfs.as_deref(),
            self.cmdline.as_deref(),
        )?;
        bz_image.init(&mut memory, self.memory_size, vcpus.get_mut(0).unwrap())?;

        self.arch_init(&mut memory, &vm_fd)?;

        let vm = Vm {
            memory,
            memory_size: self.memory_size,
            vcpus,
            devices,
        };

        Ok(vm)
    }
}

impl Vm {
    fn run_vcpu(&mut self, i: usize) -> anyhow::Result<()> {
        let vcpu = &mut self.vcpus[i];

        loop {
            // trace!("al: {}", vcpu.get_regs()?.rax & 0xFF);

            let r = vcpu.vcpu_fd.run();

            // trace!("{:?}", r);
            match r? {
                VcpuExit::IoOut(port, data) => {
                    self.devices.io_out(port, data)?;
                }
                VcpuExit::IoIn(port, data) => {
                    self.devices.io_in(port, data)?;
                }
                VcpuExit::MmioRead(_, _) => {
                    // Ignore
                }
                VcpuExit::MmioWrite(_, _) => {
                    // Ignore
                }
                VcpuExit::Unknown => todo!(),
                VcpuExit::Exception => todo!(),
                VcpuExit::Hypercall(_) => todo!(),
                VcpuExit::Debug(_) => {}
                VcpuExit::Hlt => {
                    // warn!("hlt");
                    // todo!()
                }
                VcpuExit::IrqWindowOpen => todo!(),
                VcpuExit::Shutdown => todo!(),
                VcpuExit::FailEntry(_, _) => todo!(),
                VcpuExit::Intr => todo!(),
                VcpuExit::SetTpr => todo!(),
                VcpuExit::TprAccess => todo!(),
                VcpuExit::S390Sieic => todo!(),
                VcpuExit::S390Reset => todo!(),
                VcpuExit::Dcr => todo!(),
                VcpuExit::Nmi => todo!(),
                VcpuExit::InternalError => {
                    let kvm_run = vcpu.vcpu_fd.get_kvm_run();
                    unsafe {
                        error!(?kvm_run.__bindgen_anon_1.internal, "InternalError");
                    }
                    panic!();
                }
                VcpuExit::Osi => todo!(),
                VcpuExit::PaprHcall => todo!(),
                VcpuExit::S390Ucontrol => todo!(),
                VcpuExit::Watchdog => todo!(),
                VcpuExit::S390Tsch => todo!(),
                VcpuExit::Epr => todo!(),
                VcpuExit::SystemEvent(_, _) => todo!(),
                VcpuExit::S390Stsi => todo!(),
                VcpuExit::IoapicEoi(_) => todo!(),
                VcpuExit::Hyperv => todo!(),
                VcpuExit::X86Rdmsr(_) => todo!(),
                VcpuExit::X86Wrmsr(_) => todo!(),
                VcpuExit::MemoryFault { .. } => todo!(),
                VcpuExit::Unsupported(_) => todo!(),
            }
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.run_vcpu(0)?;

        Ok(())
    }
}
