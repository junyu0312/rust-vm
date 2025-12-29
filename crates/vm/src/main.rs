#![deny(warnings)]
use clap::Parser;
use rust_vm::command::Command;
use rust_vm::kvm::vm::create_kvm_vm;
use tracing::debug;
use tracing_subscriber::EnvFilter;

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

    create_kvm_vm(args)?;

    Ok(())
}
