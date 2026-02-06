use std::path::PathBuf;
use std::slice::Iter;

use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::virt::Virt;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Result;

pub struct X86_64BootLoader {}

impl<V> BootLoaderBuilder<V> for X86_64BootLoader
where
    V: Virt,
{
    fn new(_kernel: PathBuf, _initramfs: Option<PathBuf>, _cmdline: Option<String>) -> Self {
        todo!()
    }
}

impl<V> BootLoader<V> for X86_64BootLoader
where
    V: Virt,
{
    fn load(
        &self,
        _virt: &mut V,
        _memory: &mut MemoryAddressSpace<V::Memory>,
        _irq_chip: &V::Irq,
        _devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<()> {
        todo!()
    }
}
