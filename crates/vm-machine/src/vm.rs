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
            let serial_node = fdt.begin_node("uart@09000000")?;
            fdt.property_string("compatible", "ns16550a")?;
            fdt.property_array_u64("reg", &[0x09000000, 0x1000])?;
            fdt.property_u32("clock-frequency", 24000000)?;
            fdt.property_u32("current-speed", 115200)?;
            fdt.end_node(serial_node)?;
        }

        // {
        //     let gic_node = fdt.begin_node("interrupt-controller@8000000")?;
        //     fdt.property_string("compatible", "arm,gic-v3")?;
        //     fdt.property_null("interrupt-controller")?;
        //     fdt.property_u32("#interrupt-cells", 3)?;
        //     fdt.property_array_u64(
        //         "reg",
        //         &[
        //             0x08000000, 0x10000, // GICD
        //             0x080a0000, 0x200000, // GICR
        //         ],
        //     )?;
        //     fdt.property_u32("#address-cells", 2)?;
        //     fdt.property_u32("#size-cells", 2)?;
        //     fdt.end_node(gic_node)?;
        // }

        {
            let timer_node = fdt.begin_node("timer")?;
            fdt.property_string("compatible", "arm,armv8-timer")?;
            // fdt.property_string("interrupt-parent", "interrupt-controller@8000000")?;
            // GIC PPI interrupts
            // <type irq flags>
            // type: 1 = PPI
            // flags: 4 = IRQ_TYPE_LEVEL_HIGH
            fdt.property_array_u32(
                "interrupts",
                &[
                    1, 13, 4, // CNTPNS (EL1 physical timer)
                    1, 14, 4, // CNTPS  (secure timer)
                    1, 11, 4, 1, 10, 4, // CNTV   (virtual timer)
                ],
            )?;
            fdt.end_node(timer_node)?;
        }

        {
            let chosen_node = fdt.begin_node("chosen")?;
            let bootargs = "console=ttyS0,115200 earlycon=uart8250,mmio,0x09000000,115200";
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
