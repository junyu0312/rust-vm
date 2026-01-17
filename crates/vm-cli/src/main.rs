#![deny(warnings)]

use clap::Parser;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use vm_machine::kvm::builder::create_kvm_vm;

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

    create_kvm_vm(
        args.cpus,
        args.memory << 30,
        &args.kernel,
        args.initramfs.as_deref(),
        args.cmdline.as_deref(),
    )?;

    Ok(())
}
