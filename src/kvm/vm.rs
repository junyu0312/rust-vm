use tracing::debug;

use crate::command::Command;
use crate::kvm::ioctl::*;
use crate::kvm::vcpu::create_vcpus;

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

    let vm_fd = kvm_create_vm(kvm_fd)?;
    debug!(vm_fd);

    let vcpus = create_vcpus(vm_fd, command.cpus)?;
    debug!(?vcpus);

    for vcpu in vcpus {
        vcpu.run()?;
    }

    Ok(())
}
