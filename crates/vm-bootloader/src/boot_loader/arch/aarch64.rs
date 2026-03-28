use std::path::PathBuf;
use std::slice::Iter;

use vm_core::arch::Arch;
use vm_core::arch::aarch64::layout::AArch64Layout;
use vm_core::arch::aarch64::vcpu::AArch64Vcpu;
use vm_core::arch::irq::InterruptController;
use vm_core::arch::layout::MemoryLayout;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::virt::Virt;
use vm_fdt::FdtWriter;
use vm_mm::manager::MemoryAddressSpace;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Error;
use crate::boot_loader::Result;
use crate::initrd_loader::InitrdLoader;
use crate::kernel_loader::KernelLoader;
use crate::kernel_loader::linux::image::AArch64BootParams;
use crate::kernel_loader::linux::image::Image;

const IRQ_TYPE_LEVEL_LOW: u32 = 0x00000008;

pub struct AArch64BootLoader {
    kernel: PathBuf,
    initrd: Option<PathBuf>,
    cmdline: Option<String>,
}

impl AArch64BootLoader {
    fn load_image(&self, layout: &mut AArch64Layout, memory: &MemoryAddressSpace) -> Result<()> {
        let image =
            Image::new(&self.kernel).map_err(|err| Error::LoadKernelFailed(err.to_string()))?;

        let boot_params = AArch64BootParams {
            ram_base: layout.get_ram_base(),
            ram_size: layout.get_ram_size()?,
        };
        let load_result = image
            .load(&boot_params, memory)
            .map_err(|err| Error::LoadKernelFailed(err.to_string()))?;

        layout.set_kernel(
            load_result.kernel_start,
            load_result.kernel_len,
            load_result.start_pc,
        )?;

        Ok(())
    }

    fn load_initrd(&self, layout: &mut AArch64Layout, memory: &MemoryAddressSpace) -> Result<()> {
        let Some(initrd) = self.initrd.as_deref() else {
            return Ok(());
        };

        let loader =
            InitrdLoader::new(initrd).map_err(|err| Error::LoadInitrdFailed(err.to_string()))?;

        let addr = layout.get_initrd_start();

        let result = loader
            .load(addr, memory)
            .map_err(|err| Error::LoadInitrdFailed(err.to_string()))?;

        assert_eq!(result.initrd_start, addr);
        layout.set_initrd_len(result.initrd_len)?;

        Ok(())
    }

    fn load_dtb(
        &self,
        layout: &mut AArch64Layout,
        memory: &MemoryAddressSpace,
        dtb: Vec<u8>,
    ) -> Result<()> {
        let dtb_start = layout.get_dtb_start();

        if !dtb_start.is_multiple_of(8) {
            return Err(Error::LoadDtbFailed(
                "dtb must be placed on an 8-byte boundary".to_string(),
            ));
        }

        if dtb.len() >= (2 << 20) {
            return Err(Error::LoadDtbFailed("dtb too large".to_string()));
        }

        memory
            .copy_from_slice(dtb_start, &dtb)
            .map_err(|_| Error::LoadDtbFailed("failed to copy".to_string()))?;

        layout.set_dtb_len(dtb.len())?;

        Ok(())
    }

    fn generate_dtb(
        &self,
        layout: &AArch64Layout,
        vcpus: usize,
        irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<Vec<u8>> {
        let mut fdt = FdtWriter::new()?;
        let root_node = fdt.begin_node("")?;

        fdt.property_string("compatible", "linux,virt")?;
        fdt.property_u32("#address-cells", 2)?;
        fdt.property_u32("#size-cells", 2)?;

        {
            let memory_node = fdt.begin_node(&format!("memory@{:08x}", layout.get_ram_base()))?;
            fdt.property_string("device_type", "memory")?;
            fdt.property_array_u64("reg", &[layout.get_ram_base(), layout.get_ram_size()?])?;
            fdt.end_node(memory_node)?;
        }

        {
            let cpu_node = fdt.begin_node("cpus")?;
            fdt.property_u32("#address-cells", 1)?;
            fdt.property_u32("#size-cells", 0)?;
            for i in 0..vcpus {
                let cpu_node = fdt.begin_node(&format!("cpu@{}", i))?;
                fdt.property_string("device_type", "cpu")?;
                fdt.property_string("compatible", "arm,cortex-a72")?;
                fdt.property_u32("reg", i as u32)?;
                if vcpus > 1 {
                    fdt.property_string("enable-method", "psci")?;
                }
                fdt.end_node(cpu_node)?;
            }
            fdt.end_node(cpu_node)?;
        }

        {
            let psci_node = fdt.begin_node("psci")?;
            fdt.property_string_list(
                "compatible",
                vec!["arm,psci-0.2".to_string(), "arm,psci".to_string()],
            )?;
            fdt.property_string("method", "smc")?;
            fdt.property_u32("cpu_suspend", 0x84000001)?;
            fdt.property_u32("cpu_off", 0x84000002)?;
            fdt.property_u32("cpu_on", 0x84000003)?;

            fdt.end_node(psci_node)?;
        }

        let irq_phandle = irq_chip
            .write_device_tree(&mut fdt)
            .map_err(|err| Error::LoadDtbFailed(err.to_string()))?;

        {
            let soc_node = fdt.begin_node("soc")?;
            fdt.property_string("compatible", "simple-bus")?;
            fdt.property_u32("#address-cells", 2)?;
            fdt.property_u32("#size-cells", 2)?;
            fdt.property_u32("interrupt-parent", irq_phandle as u32)?;
            fdt.property_null("ranges")?;

            {
                let timer_node = fdt.begin_node("timer")?;
                fdt.property_string("compatible", "arm,armv8-timer")?;
                fdt.property_array_u32(
                    "interrupts",
                    &[
                        1,
                        13,
                        IRQ_TYPE_LEVEL_LOW,
                        1,
                        14,
                        IRQ_TYPE_LEVEL_LOW,
                        1,
                        11,
                        IRQ_TYPE_LEVEL_LOW,
                        1,
                        10,
                        IRQ_TYPE_LEVEL_LOW,
                    ],
                )?;
                fdt.end_node(timer_node)?;
            }

            for device in devices {
                device.generate_dt(&mut fdt)?;
            }

            fdt.end_node(soc_node)?;
        }

        {
            let chosen_node = fdt.begin_node("chosen")?;
            fdt.property_u32("stdout-path", 2)?;
            if let Some(cmdline) = &self.cmdline {
                fdt.property_string("bootargs", cmdline)?;
            }
            if self.initrd.is_some() {
                fdt.property_u64("linux,initrd-start", layout.get_initrd_start())?;
                fdt.property_u64(
                    "linux,initrd-end",
                    layout.get_initrd_start() + layout.get_initrd_len()? as u64,
                )?;
            }

            fdt.end_node(chosen_node)?;
        }

        fdt.end_node(root_node)?;

        Ok(fdt.finish()?)
    }
}

impl<V> BootLoaderBuilder<V> for AArch64BootLoader
where
    V: Virt,
    V::Vcpu: AArch64Vcpu,
    V::Arch: Arch<Layout = AArch64Layout>,
{
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self {
        AArch64BootLoader {
            kernel,
            initrd: initramfs,
            cmdline,
        }
    }
}

impl<V> BootLoader<V> for AArch64BootLoader
where
    V: Virt,
    V::Vcpu: AArch64Vcpu,
    V::Arch: Arch<Layout = AArch64Layout>,
{
    fn load(
        &self,
        virt: &mut V,
        memory: &MemoryAddressSpace,
        irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn MmioDevice>>,
    ) -> Result<()> {
        {
            let layout = virt.get_layout_mut();

            self.load_image(layout, memory)?;
            self.load_initrd(layout, memory)?;
        }

        {
            let vcpus = virt.get_vcpu_number();
            let layout = virt.get_layout_mut();

            let dtb = self.generate_dtb(layout, vcpus, irq_chip, devices)?;
            self.load_dtb(layout, memory, dtb)?;
        }

        let layout = virt.get_layout();
        layout.validate()?;

        Ok(())
    }
}
