#![deny(warnings)]

use clap::Parser;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use vm_core::virt::kvm::KvmVirt;
use vm_machine::vm::Vm;
use vm_machine::vm::VmBuilder;

use crate::cmd::Command;

mod cmd;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .init();

    let args = Command::parse();
    debug!(?args);

    let mut vm: Vm<KvmVirt> = VmBuilder {
        memory_size: args.memory << 30,
        vcpus: args.cpus,
        kernel: args.kernel,
        initramfs: args.initramfs,
        cmdline: args.cmdline,
    }
    .build()?;

    vm.run()?;

    Ok(())
}
