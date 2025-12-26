#![deny(warnings)]
use clap::Parser;
use rust_vm::command::Command;
use rust_vm::kvm::device::open_kvm;
use rust_vm::kvm::vm::create_vm;
use tracing::debug;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Command::parse();
    debug!(?args);

    let kvm_fd = open_kvm()?;
    debug!(kvm_fd);

    let vm_fd = create_vm(kvm_fd)?;
    debug!(vm_fd);

    Ok(())
}
