use tokio::sync::Mutex;
use vm_bootloader::boot_loader::BootLoader;
use vm_bootloader::boot_loader::BootLoaderBuilder;
use vm_bootloader::boot_loader::arch::aarch64::AArch64BootLoader;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::vcpu_manager::VcpuManager;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;

use crate::bootloader::error::BootloaderError;
use crate::device::device_manager_v2::DeviceManagerV2;
use crate::vm::config::VmConfig;

pub async fn install_bootloader(
    vm_config: &VmConfig,
    vcpu_manager: &Mutex<VcpuManager>,
    ram_allocator: &mut RangeAllocator<u64>,
    memory_address_space: &MemoryAddressSpace,
    irq_chip: &dyn InterruptController,
    device_manager: &DeviceManagerV2,
) -> Result<(), BootloaderError> {
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
            ram_allocator,
            memory_address_space,
            irq_chip,
            device_manager.iter(),
        )
        .await?;

    Ok(())
}
