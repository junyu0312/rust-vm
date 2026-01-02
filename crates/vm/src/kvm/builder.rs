use anyhow::Context;
use kvm_ioctls::Kvm;
use tracing::debug;
use tracing::info;

use crate::arch::x86::bios::Bios;
use crate::bootable::Bootable;
use crate::bootable::linux::x86_64::bzimage::BzImage;
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

    vm.init_mm(command.memory << 30)
        .context("Failed to init mm")?;

    let bz_image = BzImage::new(&command.kernel, None, command.cmdline.as_deref())?;
    bz_image.init(&mut vm)?;

    let bios = Bios;
    bios.init(&mut vm)?;

    vm.init_device()?;

    vm.run().context("Failed to run")?;

    info!("Vm exits");

    Ok(())
}
