use std::path::Path;
use std::path::PathBuf;
use std::slice::Iter;

use async_trait::async_trait;
use vm_core::arch::aarch64::layout::DTB_START;
use vm_core::arch::aarch64::layout::INITRD_START;
use vm_core::arch::aarch64::layout::RAM_BASE;
use vm_core::arch::irq::InterruptController;
use vm_core::cpu::vcpu::Vcpu;
use vm_core::device::Device;
use vm_fdt::FdtWriter;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Error;
use crate::boot_loader::Result;
use crate::initrd_loader::InitrdLoadResult;
use crate::initrd_loader::InitrdLoader;
use crate::kernel_loader::linux::aarch64::image::AArch64BootParams;
use crate::kernel_loader::linux::aarch64::image::Image;
use crate::kernel_loader::linux::aarch64::image::LoadResult;

const IRQ_TYPE_LEVEL_LOW: u32 = 0x00000008;

pub struct AArch64BootLoader {
    kernel: PathBuf,
    initrd: Option<PathBuf>,
    cmdline: Option<String>,
}

impl AArch64BootLoader {
    fn load_initrd(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        initrd: &Path,
    ) -> Result<InitrdLoadResult> {
        let result = InitrdLoader::new(initrd)?.load(ram_allocator, memory, INITRD_START)?;

        Ok(result)
    }

    fn load_dtb(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        dtb: Vec<u8>,
    ) -> Result<u64> {
        let dtb_start = DTB_START;

        if !dtb_start.is_multiple_of(8) {
            return Err(Error::LoadDtbFailed(
                "dtb must be placed on an 8-byte boundary".to_string(),
            ));
        }

        if dtb.len() >= (2 << 20) {
            return Err(Error::LoadDtbFailed("dtb too large".to_string()));
        }

        ram_allocator.reserve(dtb_start, dtb.len())?;
        memory
            .copy_from_slice(dtb_start, &dtb)
            .map_err(|_| Error::LoadDtbFailed("failed to copy".to_string()))?;

        Ok(dtb_start)
    }

    fn generate_dtb(
        &self,
        ram_size: u64,
        initrd_load_result: Option<InitrdLoadResult>,
        vcpus: usize,
        irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<Vec<u8>> {
        let mut fdt = FdtWriter::new()?;
        let root_node = fdt.begin_node("")?;

        fdt.property_string("compatible", "linux,virt")?;
        fdt.property_u32("#address-cells", 2)?;
        fdt.property_u32("#size-cells", 2)?;

        {
            let memory_node = fdt.begin_node(&format!("memory@{:08x}", RAM_BASE))?;
            fdt.property_string("device_type", "memory")?;
            fdt.property_array_u64("reg", &[RAM_BASE, ram_size])?;
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
                if let Some(mmio_device) = device.support_mmio_transport() {
                    mmio_device.generate_dt(&mut fdt)?;
                }
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
                fdt.property_u64("linux,initrd-start", INITRD_START)?;
                fdt.property_u64(
                    "linux,initrd-end",
                    initrd_load_result.as_ref().unwrap().initrd_start
                        + initrd_load_result.as_ref().unwrap().initrd_len as u64,
                )?;
            }

            fdt.end_node(chosen_node)?;
        }

        fdt.end_node(root_node)?;

        Ok(fdt.finish()?)
    }

    fn load_image(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
    ) -> Result<LoadResult> {
        let boot_params = AArch64BootParams { ram_base: RAM_BASE };
        let load_result = Image::new(&self.kernel)?.load(ram_allocator, memory, &boot_params)?;

        Ok(load_result)
    }
}

impl BootLoaderBuilder for AArch64BootLoader {
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self {
        AArch64BootLoader {
            kernel,
            initrd: initramfs,
            cmdline,
        }
    }
}

#[async_trait]
impl BootLoader for AArch64BootLoader {
    async fn load(
        &self,
        ram_size: u64,
        vcpus: usize,
        boot_vcpu: &mut Vcpu,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<()> {
        let initrd_loader = if let Some(initrd) = &self.initrd {
            let load_result = self.load_initrd(ram_allocator, memory, initrd)?;
            Some(load_result)
        } else {
            None
        };

        let dtb_start = {
            let dtb = self.generate_dtb(ram_size, initrd_loader, vcpus, irq_chip, devices)?;
            self.load_dtb(ram_allocator, memory, dtb)?
        };

        let kernel_loader = self.load_image(ram_allocator, memory)?;

        boot_vcpu
            .setup_vcpu(kernel_loader.start_pc, dtb_start)
            .await?;

        Ok(())
    }
}
