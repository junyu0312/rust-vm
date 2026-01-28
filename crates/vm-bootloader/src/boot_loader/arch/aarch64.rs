use std::path::PathBuf;
use std::slice::Iter;

use vm_core::arch::Arch;
use vm_core::device::Device;
use vm_core::irq::arch::aarch64::AArch64IrqChip;
use vm_core::layout::MemoryLayout;
use vm_core::layout::aarch64::AArch64Layout;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::vcpu::arch::aarch64::AArch64Vcpu;
use vm_core::vcpu::arch::aarch64::reg::CoreRegister;
use vm_core::vcpu::arch::aarch64::reg::cnthctl_el2::CnthctlEl2;
use vm_core::vcpu::arch::aarch64::reg::sctlr_el1::SctlrEl1;
use vm_core::virt::Virt;
use vm_fdt::FdtWriter;

use crate::boot_loader::BootLoader;
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
    pub fn new(kernel: PathBuf, initrd: Option<PathBuf>, cmdline: Option<String>) -> Self {
        AArch64BootLoader {
            kernel,
            initrd,
            cmdline,
        }
    }
}

impl AArch64BootLoader {
    fn load_image<C>(
        &self,
        layout: &mut AArch64Layout,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<()>
    where
        C: MemoryContainer,
    {
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

    fn load_initrd<C>(
        &self,
        layout: &mut AArch64Layout,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<()>
    where
        C: MemoryContainer,
    {
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

    fn load_dtb<C>(
        &self,
        layout: &mut AArch64Layout,
        memory: &mut MemoryAddressSpace<C>,
        dtb: Vec<u8>,
    ) -> Result<()>
    where
        C: MemoryContainer,
    {
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
            .copy_from_slice(dtb_start, &dtb, dtb.len())
            .map_err(|_| Error::LoadDtbFailed("failed to copy".to_string()))?;

        layout.set_dtb_len(dtb.len())?;

        Ok(())
    }

    fn setup_boot_cpu<C>(&self, dtb_start: u64, start_pc: u64, vcpus: &mut [C]) -> Result<()>
    where
        C: AArch64Vcpu,
    {
        let mut setup = || {
            let boot_cpu = vcpus.get_mut(0).unwrap();

            {
                // Setup general-purpose register
                boot_cpu.set_core_reg(CoreRegister::X0, dtb_start)?;
                boot_cpu.set_core_reg(CoreRegister::X1, 0)?;
                boot_cpu.set_core_reg(CoreRegister::X2, 0)?;
                boot_cpu.set_core_reg(CoreRegister::X3, 0)?;
                boot_cpu.set_core_reg(CoreRegister::PC, start_pc)?;
            }

            {
                // CPU mode

                let mut pstate = boot_cpu.get_core_reg(CoreRegister::PState)?;
                pstate |= 0x03C0; // DAIF
                pstate &= !0xf; // Clear low 4 bits
                pstate |= 0x0005; // El1h
                boot_cpu.set_core_reg(CoreRegister::PState, pstate)?;

                // more, non secure el1
                if false {
                    todo!()
                }
            }

            {
                // Caches, MMUs

                let mut sctlr_el1 = boot_cpu.get_sctlr_el1()?;
                sctlr_el1.remove(SctlrEl1::M); // Disable MMU
                sctlr_el1.remove(SctlrEl1::I); // Disable I-cache
                boot_cpu.set_sctlr_el1(sctlr_el1)?;
            }

            {
                // Architected timers

                if false {
                    todo!(
                        "CNTFRQ must be programmed with the timer frequency and CNTVOFF must be programmed with a consistent value on all CPUs."
                    );
                }

                if false {
                    // MacOS get panic, should we enable this in Linux?
                    let mut cnthctl_el2 = boot_cpu.get_cnthctl_el2()?;
                    cnthctl_el2.insert(CnthctlEl2::EL1PCTEN); // TODO: or bit0?(https://www.kernel.org/doc/html/v5.3/arm64/booting.html)
                    boot_cpu.set_cnthctl_el2(cnthctl_el2)?;
                }
            }

            {
                // Coherency

                // Do nothing
            }

            {
                // System registers

                if false {
                    todo!()
                }
            }

            anyhow::Ok(())
        };

        setup().map_err(|err| Error::SetupBootCpuFailed(err.to_string()))?;

        Ok(())
    }

    fn generate_dtb(
        &self,
        layout: &AArch64Layout,
        vcpus: usize,
        irq_chip: &dyn AArch64IrqChip,
        devices: Iter<'_, Box<dyn Device>>,
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
                fdt.end_node(cpu_node)?;
            }
            fdt.end_node(cpu_node)?;
        }

        let irq_phandle = irq_chip.write_device_tree(&mut fdt).unwrap();

        {
            let soc_node = fdt.begin_node("soc")?;
            fdt.property_string("compatible", "simple-bus")?;
            fdt.property_u32("#address-cells", 2)?;
            fdt.property_u32("#size-cells", 2)?;
            fdt.property_u32("interrupt-parent", irq_phandle)?;
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
                if let Some(mmio_device) = device.as_mmio_device() {
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

impl<V> BootLoader<V> for AArch64BootLoader
where
    V: Virt,
    V::Vcpu: AArch64Vcpu,
    V::Irq: AArch64IrqChip,
    V::Arch: Arch<Layout = AArch64Layout>,
{
    fn load(
        &self,
        virt: &mut V,
        memory: &mut MemoryAddressSpace<V::Memory>,
        irq_chip: &V::Irq,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<()> {
        {
            let layout = virt.get_layout_mut();

            self.load_image(layout, memory)?;
            self.load_initrd(layout, memory)?;
        }

        {
            let vcpus = virt
                .get_vcpus()
                .map_err(|err| Error::SetupBootCpuFailed(err.to_string()))?
                .len();

            let layout = virt.get_layout_mut();

            let dtb = self.generate_dtb(layout, vcpus, irq_chip, devices)?;
            self.load_dtb(layout, memory, dtb)?;
        }

        {
            let dtb_start = virt.get_layout().get_dtb_start();
            let start_pc = virt.get_layout().get_start_pc()?;

            let vcpus = virt
                .get_vcpus_mut()
                .map_err(|err| Error::SetupBootCpuFailed(err.to_string()))?;

            self.setup_boot_cpu(dtb_start, start_pc, vcpus)?;
        }

        let layout = virt.get_layout();
        layout.validate()?;

        Ok(())
    }
}
