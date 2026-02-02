use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use vm_bootloader::boot_loader::BootLoader;
use vm_core::device::IoAddressSpace;
use vm_core::device::mmio::MmioLayout;
use vm_core::layout::MemoryLayout;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::mm::region::MemoryRegion;
use vm_core::virt::Virt;

use crate::device::init_device;

pub struct VmBuilder {
    pub memory_size: usize,
    pub vcpus: usize,
    pub kernel: PathBuf,
    pub initramfs: Option<PathBuf>,
    pub cmdline: Option<String>,
}

pub struct Vm<V: Virt> {
    pub(crate) memory: Arc<Mutex<MemoryAddressSpace<V::Memory>>>,
    pub(crate) virt: V,
    pub(crate) irq_chip: Arc<V::Irq>,
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
        let mut virt = V::new()?;

        let irq_chip = virt.init_irq()?;

        virt.init_vcpus(self.vcpus)?;

        let layout = virt.get_layout();
        let mmio_layout = MmioLayout::new(layout.get_mmio_start(), layout.get_mmio_len());

        let mut memory = self.init_mm(layout.get_ram_base())?;
        virt.init_memory(&mmio_layout, &mut memory, self.memory_size as u64)?;
        let memory = Arc::new(Mutex::new(memory));

        virt.post_init()?;

        let mut devices = IoAddressSpace::new(mmio_layout);
        init_device(memory.clone(), &mut devices, irq_chip.clone())?;

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
            virt,
            irq_chip,
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
        let mut memory = self.memory.lock().unwrap();

        boot_loader.load(
            &mut self.virt,
            &mut memory,
            &self.irq_chip,
            self.devices.devices(),
        )?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.virt.run(&mut self.devices)?;

        Ok(())
    }
}
