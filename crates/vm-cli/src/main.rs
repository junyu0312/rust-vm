// #![deny(warnings)]

use clap::Parser;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use vm_core::virtualization::hypervisor::Hypervisor;
use vm_vmm::vm::config::VmConfig;
use vm_vmm::vmm::Vmm;

use crate::cmd::Accel;
use crate::cmd::Command;
use crate::cmd::parse_memory;
use crate::term::term_init;

mod cmd;
mod term;

async fn build_and_run_vm(hypervisor: Box<dyn Hypervisor>, args: Command) -> anyhow::Result<()> {
    let mut vmm = Vmm::new(hypervisor);

    vmm.create_vm_from_config(VmConfig {
        memory_size: parse_memory(&args.memory)?,
        vcpus: args.cpus,
        devices: args.device.into_iter().map(Into::into).collect(),
        gdb_port: args.gdb,
        kernel: args.kernel,
        initramfs: args.initramfs,
        cmdline: args.cmdline,
    })?;

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

    let args = Command::parse();
    debug!(?args);

    let _term_backup = term_init()?;

    match args.accel {
        #[cfg(feature = "kvm")]
        Accel::Kvm => {
            todo!()
        }

        #[cfg(feature = "hvp")]
        Accel::Hvp => {
            #[cfg(not(target_arch = "aarch64"))]
            {
                bail!("hvp only supports aarch64");
            }

            #[cfg(target_arch = "aarch64")]
            {
                use vm_core::virtualization::hvp::AppleHypervisor;

                build_and_run_vm(Box::new(AppleHypervisor), args).await?;
            }
        }
    };

    Ok(())
}
