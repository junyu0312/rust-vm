use std::path::PathBuf;
use std::slice::Iter;

use async_trait::async_trait;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::vcpu::Vcpu;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_mm::manager::MemoryAddressSpace;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Result;

pub struct X86_64BootLoader {}

impl BootLoaderBuilder for X86_64BootLoader {
    fn new(_kernel: PathBuf, _initramfs: Option<PathBuf>, _cmdline: Option<String>) -> Self {
        todo!()
    }
}

#[async_trait]
impl BootLoader for X86_64BootLoader {
    async fn load(
        &self,
        _ram_size: u64,
        _vcpus: usize,
        _boot_vcpu: &mut Vcpu,
        _memory: &MemoryAddressSpace,
        _irq_chip: &dyn InterruptController,
        _devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<()> {
        todo!()
    }
}
