use vm_bootloader::boot_loader::BootLoader;
use vm_bootloader::boot_loader::BootLoaderBuilder;
use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;

use crate::bootloader::error::BootLoaderError;

pub async fn install_bootloader(
    vm_config: &VmConfig,
    vcpu_manager: &Mutex<VcpuManager>,
    memory_address_space: &MemoryAddressSpace,
) -> Result<(), BootLoaderError> {
    let bootloader = AArch64BootLoader::new(
        vm_config.kernel.clone(),
        vm_config.initramfs.clone(),
        vm_config.cmdline.clone(),
    );

    let mut vcpu_manager = vcpu_manager.lock().await;

    let boot_vcpu = vcpu_manager.get_vcpu_mut(0)?;
    bootloader
        .load(
            vm_config.memory_size as u64,
            vm_config.vcpus,
            boot_vcpu,
            &memory_address_space,
            irq_chip.as_ref(),
            device_manager.mmio_devices(),
        )
        .await?;

    Ok(())
}
