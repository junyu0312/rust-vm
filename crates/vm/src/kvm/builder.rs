use anyhow::Context;
use kvm_ioctls::Kvm;
use tracing::debug;
use tracing::info;

use crate::command::Command;
use crate::kvm::vm::KvmVm;

pub fn create_kvm_vm(command: Command) -> anyhow::Result<()> {
    let kvm = Kvm::new()?;

    let kvm_nr_vcpus = kvm.get_nr_vcpus();
    debug!(kvm_nr_vcpus);

    let kvm_max_vcpus = kvm.get_max_vcpus();
    debug!(kvm_max_vcpus);

    let kvm_max_vcpu_id = kvm.get_max_vcpu_id();

    command.validate(kvm_nr_vcpus, kvm_max_vcpus, kvm_max_vcpu_id)?;

    let mut vm = KvmVm::new(kvm)?;

    vm.init_vcpus(command.cpus)
        .context("Failed to create vcpus")?;
    vm.init_mm(command.memory).context("Failed to init mm")?;
    vm.init_device()?;

    vm.init_kernel(command.kernel.as_path(), command.cmdline)?;

    vm.run().context("Failed to run")?;

    info!("Vm exits");

    Ok(())
}
