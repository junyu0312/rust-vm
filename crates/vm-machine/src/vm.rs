use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use vm_bootloader::boot_loader::BootLoader;
use vm_core::arch::Arch;
use vm_core::device::pio::IoAddressSpace;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::mm::region::MemoryRegion;
use vm_core::virt::Virt;
use vm_fdt::FdtWriter;

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
        let mut virt = V::new()?;

        let kvm_irq = Arc::new(virt.init_irq()?);

        virt.init_vcpus(self.vcpus)?;

        let mut memory = self.init_mm(<V::Arch as Arch>::BASE_ADDRESS)?;
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
    pub fn generate_dtb(&self) -> anyhow::Result<Vec<u8>> {
        let mut fdt = FdtWriter::new()?;
        let root_node = fdt.begin_node("")?;

        fdt.property_string("compatible", "linux,virt")?;
        fdt.property_u32("#address-cells", 2)?;
        fdt.property_u32("#size-cells", 2)?;

        {
            let memory_node =
                fdt.begin_node(&format!("memory@{}", <V::Arch as Arch>::BASE_ADDRESS,))?;
            fdt.property_string("device_type", "memory")?;
            fdt.property_array_u64(
                "reg",
                &[<V::Arch as Arch>::BASE_ADDRESS, self.memory_size as u64],
            )?;
            fdt.end_node(memory_node)?;
        }

        {
            let cpu_node = fdt.begin_node("cpus")?;
            fdt.property_u32("#address-cells", 1)?;
            fdt.property_u32("#size-cells", 0)?;
            for (i, _vcpu) in self.virt.get_vcpus().iter().enumerate() {
                let cpu_node = fdt.begin_node(&format!("cpu@{}", i))?;
                fdt.property_string("device_type", "cpu")?;
                fdt.property_string("compatible", "arm,cortex-a72")?;
                fdt.property_u32("reg", i as u32)?;
                fdt.end_node(cpu_node)?;
            }
            fdt.end_node(cpu_node)?;
        }

        {
            let chosen_node = fdt.begin_node("chosen")?;
            let bootargs = "console=ttyAMA0,115200 earlycon=uart,mmio,0x09000000,115200";
            fdt.property_string("bootargs", bootargs)?;
            fdt.end_node(chosen_node)?;
        }

        fdt.end_node(root_node)?;
        let dtb = fdt.finish()?;

        anyhow::Ok(dtb)
    }

    pub fn load(&mut self, boot_loader: &dyn BootLoader<V::Memory, V::Vcpu>) -> anyhow::Result<()> {
        boot_loader.load(
            <V::Arch as Arch>::BASE_ADDRESS,
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
