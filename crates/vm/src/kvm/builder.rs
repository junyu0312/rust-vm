use std::sync::Arc;
use std::sync::mpsc;

use anyhow::Context;
use kvm_bindings::kvm_pit_config;
use kvm_ioctls::Kvm;
use tracing::debug;
use tracing::info;

use crate::arch::x86::bios::Bios;
use crate::bootable::Bootable;
use crate::bootable::linux::x86_64::bzimage::BzImage;
use crate::command::Command;
use crate::kvm::irq::KvmIRQ;
use crate::kvm::vm::KvmVm;
use crate::utils::stdin::init_stdin;

pub fn create_kvm_vm(command: Command) -> anyhow::Result<()> {
    let kvm = Kvm::new()?;

    let kvm_nr_vcpus = kvm.get_nr_vcpus();
    debug!(kvm_nr_vcpus);

    let kvm_max_vcpus = kvm.get_max_vcpus();
    debug!(kvm_max_vcpus);

    let kvm_max_vcpu_id = kvm.get_max_vcpu_id();

    command.validate(kvm_nr_vcpus, kvm_max_vcpus, kvm_max_vcpu_id)?;

    let mut vm = KvmVm::new(kvm)?;

    let kvm_irq = Arc::new(KvmIRQ::new(&vm)?);

    {
        let pit_config = kvm_pit_config::default();
        vm.vm_fd.create_pit2(pit_config).unwrap();
        // let mut pitstate = kvm_pit_state2::default();
        // Your `pitstate` manipulation here.
        // vm.set_pit2(&mut pitstate).unwrap();
    }

    vm.init_vcpus(command.cpus)
        .context("Failed to create vcpus")?;

    vm.init_mm(command.memory << 30)
        .context("Failed to init mm")?;

    let bz_image = BzImage::new(
        &command.kernel,
        command.initramfs.as_deref(),
        command.cmdline.as_deref(),
    )?;
    bz_image.init(&mut vm)?;

    let bios = Bios;
    bios.init(&mut vm)?;

    let (tx, rx) = mpsc::channel();
    init_stdin(tx);
    vm.init_device(kvm_irq, rx)?;

    vm.run().context("Failed to run")?;

    info!("Vm exits");

    Ok(())
}
