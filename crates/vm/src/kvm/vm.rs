use std::cell::OnceCell;

use anyhow::Context;
use kvm_ioctls::Kvm;
use kvm_ioctls::VmFd;
use tracing::debug;
use tracing::info;

use crate::command::Command;
use crate::kvm::vcpu::KvmVcpu;
use crate::mm::manager::MemoryRegions;

pub struct KvmVm {
    pub vm_fd: VmFd,
    pub vcpus: OnceCell<Vec<KvmVcpu>>,
    pub memory_regions: OnceCell<MemoryRegions>,
}

impl KvmVm {
    fn new(kvm: &Kvm) -> anyhow::Result<Self> {
        let vm_fd = kvm.create_vm()?;
        Ok(KvmVm {
            vm_fd,
            vcpus: Default::default(),
            memory_regions: Default::default(),
        })
    }
}

pub fn create_kvm_vm(command: Command) -> anyhow::Result<()> {
    let kvm = Kvm::new()?;

    let kvm_nr_vcpus = kvm.get_nr_vcpus();
    debug!(kvm_nr_vcpus);

    let kvm_max_vcpus = kvm.get_max_vcpus();
    debug!(kvm_max_vcpus);

    let kvm_max_vcpu_id = kvm.get_max_vcpu_id();

    command.validate(kvm_nr_vcpus, kvm_max_vcpus, kvm_max_vcpu_id)?;

    let mut vm = KvmVm::new(&kvm)?;

    vm.init_vcpus(command.cpus)
        .context("Failed to create vcpus")?;
    vm.init_mm(command.memory).context("Failed to init mm")?;

    if let Some(kernel) = command.kernel {
        vm.init_kernel(kernel.as_path())?;
    }

    vm.run().context("Failed to run")?;

    info!("Vm exits");

    Ok(())
}
