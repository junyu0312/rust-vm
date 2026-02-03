#![deny(warnings)]

use clap::Parser;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use vm_bootloader::boot_loader::BootLoaderBuilder;
use vm_core::virt::Virt;
use vm_machine::vm::VmBuilder;

use crate::cmd::Accel;
use crate::cmd::Command;
use crate::cmd::parse_memory;
use crate::term::term_init;

mod cmd;
mod term;

fn build_and_run_vm<V, Loader>(args: Command) -> anyhow::Result<()>
where
    V: Virt,
    Loader: BootLoaderBuilder<V>,
{
    let vm_builder = VmBuilder::<V>::new(parse_memory(&args.memory)?, args.cpus);
    let mut vm = vm_builder.build()?;

    let bootloader = Loader::new(args.kernel, args.initramfs, args.cmdline);
    vm.load(&bootloader)?;

    vm.run()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .init();

    let args = Command::parse();
    debug!(?args);

    let _term_backup = term_init()?;

    match args.accel {
        #[cfg(feature = "kvm")]
        Accel::Kvm => {
            #[cfg(target_arch = "aarch64")]
            {
                use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;
                use vm_core::arch::aarch64::AArch64;
                use vm_core::virt::kvm::KvmVirt;

                build_and_run_vm::<KvmVirt<AArch64>, AArch64BootLoader>(args)?;
            }

            #[cfg(target_arch = "x86_64")]
            {
                use vm_bootloader::boot_loader::arch::x86_64::X86_64BootLoader;
                use vm_core::arch::x86_64::X86_64;
                use vm_core::virt::kvm::KvmVirt;

                build_and_run_vm::<KvmVirt<X86_64>, X86_64BootLoader>(args)?;
            }
        }

        #[cfg(feature = "hvp")]
        Accel::Hvp => {
            #[cfg(not(target_arch = "aarch64"))]
            {
                bail!("hvp only supports aarch64");
            }

            #[cfg(target_arch = "aarch64")]
            {
                use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;
                use vm_core::virt::hvp::Hvp;

                build_and_run_vm::<Hvp, AArch64BootLoader>(args)?;
            }
        }
    };

    Ok(())
}
