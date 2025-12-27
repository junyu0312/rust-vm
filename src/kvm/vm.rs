use std::cell::OnceCell;

use anyhow::Context;
use nix::libc::c_int;
use tracing::debug;
use tracing::info;

use crate::command::Command;
use crate::kvm::ioctl::*;
use crate::kvm::vcpu::KvmVcpu;
use crate::mm::MemoryRegion;

#[derive(Default)]
pub struct KvmVm {
    pub vm_fd: c_int,
    pub vcpus: OnceCell<Vec<KvmVcpu>>,
    pub memory_regions: OnceCell<Vec<MemoryRegion>>,
}

impl KvmVm {
    fn new(kvm_fd: c_int) -> anyhow::Result<Self> {
        let vm_fd = kvm_create_vm(kvm_fd)?;
        Ok(KvmVm {
            vm_fd,
            ..Default::default()
        })
    }
}

pub fn create_kvm_vm(command: Command) -> anyhow::Result<()> {
    let kvm_fd = open_kvm()?;
    debug!(kvm_fd);

    let kvm_cap_nr_vcpus = kvm_cap_nr_vcpus(kvm_fd)?;
    debug!(kvm_cap_nr_vcpus);

    let kvm_cap_max_vcpus = kvm_cap_max_vcpus(kvm_fd)?;
    debug!(kvm_cap_max_vcpus);

    let kvm_cap_max_vcpu_id = kvm_cap_max_vcpu_id(kvm_fd, kvm_cap_max_vcpus)?;
    debug!(kvm_cap_max_vcpu_id);

    command.validate(kvm_cap_nr_vcpus, kvm_cap_max_vcpus, kvm_cap_max_vcpu_id)?;

    let mut vm = KvmVm::new(kvm_fd)?;

    vm.create_vcpus(command.cpus)
        .context("Failed to create vcpus")?;
    vm.init_mm(command.memory).context("Failed to init mm")?;
    vm.run().context("Failed to run")?;

    info!("Vm exits");

    Ok(())
}
