use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use vm_core::device::pio::IoAddressSpace;
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

#[allow(dead_code)]
pub struct Vm<V: Virt> {
    memory: MemoryAddressSpace<V::Memory>,
    memory_size: usize,

    virt: V,

    devices: IoAddressSpace,
}

impl VmBuilder {
    fn init_mm<C>(&self) -> anyhow::Result<MemoryAddressSpace<C>>
    where
        C: MemoryContainer,
    {
        let memory_region = MemoryRegion::new(0, self.memory_size)?;

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

        let kvm_irq = Arc::new(virt.init_irq()?);

        virt.init_vcpus(self.vcpus)?;

        #[allow(unused_mut)]
        let mut memory = self.init_mm()?;
        virt.init_memory(&mut memory)?;

        virt.post_init()?;

        let devices = init_device(kvm_irq)?;

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
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.virt.run(&mut self.devices)?;

        Ok(())
    }
}
