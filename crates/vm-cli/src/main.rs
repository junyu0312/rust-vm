#![deny(warnings)]

use clap::Parser;
use tracing::debug;
use tracing_subscriber::EnvFilter;
use vm_machine::vm::Vm;
use vm_machine::vm::VmBuilder;

use crate::cmd::Accel;
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

    match args.accel {
        #[cfg(feature = "kvm")]
        Accel::Kvm => {
            #[cfg(target_arch = "aarch64")]
            {
                use vm_core::arch::aarch64::AArch64;
                use vm_core::virt::kvm::KvmVirt;

                let mut vm: Vm<KvmVirt<AArch64>> = VmBuilder {
                    memory_size: args.memory << 30,
                    vcpus: args.cpus,
                    kernel: args.kernel,
                    initramfs: args.initramfs,
                    cmdline: args.cmdline,
                }
                .build()?;

                vm.run()?;
            }

            #[cfg(target_arch = "x86_64")]
            {
                let mut vm: Vm<KvmVirt<X86_64>> = VmBuilder {
                    memory_size: args.memory << 30,
                    vcpus: args.cpus,
                    kernel: args.kernel,
                    initramfs: args.initramfs,
                    cmdline: args.cmdline,
                }
                .build()?;

                vm.run()?;
            }
        }
        #[cfg(feature = "hvp")]
        Accel::Hvp => {
            use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;
            use vm_core::virt::hvp::Hvp;

            let mut vm: Vm<Hvp> = VmBuilder {
                memory_size: args.memory << 30,
                vcpus: args.cpus,
                kernel: args.kernel.clone(),
                initramfs: args.initramfs.clone(),
                cmdline: args.cmdline.clone(),
            }
            .build()?;

            let dtb = vm.generate_dtb(args.cmdline.as_deref())?;

            let bootloader = AArch64BootLoader::new(args.kernel, args.initramfs, dtb);
            vm.load(&bootloader)?;

            vm.run()?;
        }
    };

    Ok(())
}
