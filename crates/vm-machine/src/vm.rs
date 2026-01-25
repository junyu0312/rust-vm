use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use vm_bootloader::boot_loader::BootLoader;
use vm_core::arch::Arch;
use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioLayout;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::mm::region::MemoryRegion;
use vm_core::virt::Virt;

use crate::device::init_device;

pub mod dtb;

pub struct VmBuilder {
    pub memory_size: usize,
    pub vcpus: usize,
    pub kernel: PathBuf,
    pub initramfs: Option<PathBuf>,
    pub cmdline: Option<String>,
}

#[allow(dead_code)]
pub struct Vm<V: Virt> {
    pub(crate) memory: MemoryAddressSpace<V::Memory>,
    pub(crate) memory_size: usize,

    pub(crate) virt: V,

    pub(crate) devices: IoAddressSpace,
}

impl VmBuilder {
    fn init_mm<C>(&self, ram_base: u64) -> anyhow::Result<MemoryAddressSpace<C>>
    where
        C: MemoryContainer,
    {
        let memory_region = MemoryRegion::new(ram_base, self.memory_size)?;

        let mut memory_regions = MemoryAddressSpace::default();
        memory_regions
            .try_insert(memory_region)
            .map_err(|_| anyhow!("Failed to insert memory_region"))?;

        Ok(memory_regions)
    }

    pub fn build<V>(&self) -> anyhow::Result<Vm<V>>
    where
        V: Virt,
    {
        let mmio_layout =
            MmioLayout::new(<V::Arch as Arch>::MMIO_START, <V::Arch as Arch>::MMIO_LEN);

        let mut virt = V::new()?;

        let kvm_irq = Arc::new(virt.init_irq()?);

        virt.init_vcpus(self.vcpus)?;

        let mut memory = self.init_mm(<V::Arch as Arch>::BASE_ADDRESS)?;
        virt.init_memory(&mmio_layout, &mut memory)?;

        virt.post_init()?;

        let devices = init_device(mmio_layout, kvm_irq)?;

        /*
        #[cfg(target_arch = "x86_64")]
        {
            use vm_bootloader::BootLoader;
            use vm_bootloader::linux::bzimage::BzImage;

            use crate::firmware::bios::Bios;

            let bz_image = BzImage::new(
                &self.kernel,
                self.initramfs.as_deref(),
                self.cmdline.as_deref(),
            )?;
            let vcpu0 = virt.get_vcpu_mut(0)?.unwrap();
            bz_image.init(&mut memory, self.memory_size, vcpu0)?;

            {
                let bios = Bios;
                bios.init(&mut memory, self.memory_size)?;
            }
        }
        */

        let vm = Vm {
            memory,
            memory_size: self.memory_size,
            virt,
            devices,
        };

        Ok(vm)
    }
}

impl<V> Vm<V>
where
    V: Virt,
{
    pub fn load(&mut self, boot_loader: &dyn BootLoader<V>) -> anyhow::Result<()> {
        boot_loader.load(
            <V::Arch as Arch>::BASE_ADDRESS,
            self.memory_size as u64,
            &mut self.memory,
            self.virt.get_vcpus_mut()?,
        )?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.virt.run(&mut self.devices)?;

        Ok(())
    }
}
