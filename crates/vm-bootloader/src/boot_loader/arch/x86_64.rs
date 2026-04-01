use std::path::PathBuf;
use std::slice::Iter;

use vm_core::arch::irq::InterruptController;
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

impl BootLoader for X86_64BootLoader {
    fn load(
        &self,
        _ram_size: u64,
        _vcpus: usize,
        _memory: &MemoryAddressSpace,
        _irq_chip: &dyn InterruptController,
        _devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<u64> {
        todo!()
    }
}
