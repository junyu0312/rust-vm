use vm_bootloader::boot_loader::BootLoaderBuilder;
use vm_bootloader::boot_loader::arch::x86_64::X86_64BootLoader;

use crate::bootloader::error::BootloaderError;
use crate::vm::config::VmConfig;

pub fn install_bootloader(vm_config: &VmConfig) -> Result<(), BootloaderError> {
    let _bootloader = X86_64BootLoader::new(
        vm_config.kernel.clone(),
        vm_config.initramfs.clone(),
        vm_config.cmdline.clone(),
    );

    todo!()
}
