#![deny(warnings)]

use std::fs;

use clap::Parser;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use vm_core::virtualization::hypervisor::Hypervisor;
use vm_vmm::vmm::Vmm;

use crate::cmd::Cli;
use crate::cmd::Command;
use crate::cmd::json::CreateArgs;
use crate::term::term_init;

mod cmd;
mod error;
mod term;

fn build_hypervisor() -> anyhow::Result<Box<dyn Hypervisor>> {
    cfg_select! {
        all(target_arch = "aarch64", feature = "hvp") => {
            Ok(Box::new(vm_core::virtualization::hvp::AppleHypervisor))
        }
        feature = "kvm" => {
            Ok(Box::new(vm_core::virtualization::kvm::KvmHypervisor::new()?))
        }
        _ => panic!(),
    }
}

async fn build_and_run_vm(args: Command) -> anyhow::Result<()> {
    let hypervisor = build_hypervisor()?;

    let mut vmm = Vmm::new(hypervisor);

    match args {
        Command::Json { path } => {
            let json = fs::read(path)?;
            let json = serde_json::from_slice::<CreateArgs>(&json)?;

            debug!("create vm from json: {:?}", json);

            vmm.create_vm_from_config(json.try_into()?).await?;

            vmm.try_boot().await?;
        }
        Command::Snapshot { path } => {
            debug!("import snapshot from {:?}", path);

            vmm.create_vm_from_snapshot(&path).await?;

            debug!("vm is booting");

            vmm.try_boot().await?;
        }
    }

    vmm.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .init();

    let args = Cli::parse();
    debug!(?args);

    let _term_backup = term_init()?;

    build_and_run_vm(args.command).await?;

    Ok(())
}
