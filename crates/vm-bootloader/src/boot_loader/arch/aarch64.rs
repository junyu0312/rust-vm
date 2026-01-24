use std::cell::OnceCell;
use std::path::PathBuf;

use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::vcpu::arch::aarch64::AArch64Vcpu;
use vm_core::vcpu::arch::aarch64::reg::CoreRegister;
use vm_core::vcpu::arch::aarch64::reg::cnthctl_el2::CnthctlEl2;
use vm_core::vcpu::arch::aarch64::reg::sctlr_el1::SctlrEl1;

use crate::boot_loader::BootLoader;
use crate::boot_loader::Error;
use crate::boot_loader::Result;
use crate::kernel_loader::KernelLoader;
use crate::kernel_loader::linux::image::AArch64BootParams;
use crate::kernel_loader::linux::image::Image;

struct AArch64Layout {
    ram_base: u64,
    dtb_start: OnceCell<u64>,
    dtb_end: OnceCell<u64>,
    kernel_start: OnceCell<u64>,
    kernel_end: OnceCell<u64>,
    start_pc: OnceCell<u64>,
}

impl AArch64Layout {
    fn new(ram_base: u64) -> AArch64Layout {
        AArch64Layout {
            ram_base,
            dtb_start: OnceCell::new(),
            dtb_end: OnceCell::new(),
            kernel_start: OnceCell::new(),
            kernel_end: OnceCell::new(),
            start_pc: OnceCell::new(),
        }
    }

    fn finalize(self) -> Result<()> {
        let dtb_start = self.dtb_start.get().unwrap();
        let kernel_end = self.kernel_end.get().unwrap();

        if kernel_end > dtb_start {
            return Err(Error::MemoryOverlap);
        }

        Ok(())
    }
}

pub struct AArch64BootLoader {
    kernel: PathBuf,
    initrd: Option<PathBuf>,
    cmdline: Option<String>,
    dtb: Vec<u8>,
}

impl AArch64BootLoader {
    pub fn new(
        kernel: PathBuf,
        initrd: Option<PathBuf>,
        cmdline: Option<String>,
        dtb: Vec<u8>,
    ) -> Self {
        AArch64BootLoader {
            kernel,
            initrd,
            cmdline,
            dtb,
        }
    }
}

impl AArch64BootLoader {
    fn load_kernel<C>(
        &self,
        layout: &mut AArch64Layout,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<()>
    where
        C: MemoryContainer,
    {
        let image = Image::new(
            &self.kernel,
            self.initrd.as_deref(),
            self.cmdline.as_deref(),
        )
        .map_err(|err| Error::LoadKernelFailed(err.to_string()))?;

        let boot_params = AArch64BootParams {
            ram_base: layout.ram_base,
        };
        let load_result = image
            .load(&boot_params, memory)
            .map_err(|err| Error::LoadKernelFailed(err.to_string()))?;

        layout.kernel_start.set(load_result.kernel_start).unwrap();
        layout.kernel_end.set(load_result.kernel_end).unwrap();
        layout.start_pc.set(load_result.start_pc).unwrap();

        Ok(())
    }

    fn load_dtb<C>(
        &self,
        layout: &mut AArch64Layout,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<()>
    where
        C: MemoryContainer,
    {
        let dtb_start = layout.kernel_end.get().unwrap().next_multiple_of(8);
        let dtb_end = dtb_start + self.dtb.len() as u64;

        if !dtb_start.is_multiple_of(8) {
            return Err(Error::LoadDtbFailed(
                "dtb must be placed on an 8-byte boundary".to_string(),
            ));
        }

        if self.dtb.len() >= (2 << 20) {
            return Err(Error::LoadDtbFailed("dtb too large".to_string()));
        }

        memory
            .copy_from_slice(dtb_start, &self.dtb, self.dtb.len())
            .map_err(|_| Error::LoadDtbFailed("failed to copy".to_string()))?;

        layout.dtb_start.set(dtb_start).unwrap();
        layout.dtb_end.set(dtb_end).unwrap();

        Ok(())
    }

    fn setup_boot_cpu<C>(&self, layout: &mut AArch64Layout, vcpus: &mut [C]) -> Result<()>
    where
        C: AArch64Vcpu,
    {
        let mut setup = || {
            let boot_cpu = vcpus.get_mut(0).unwrap();

            {
                // Setup general-purpose register

                let dtb_start = layout.dtb_start.get().unwrap();
                let kernel_start = layout.kernel_start.get().unwrap();
                boot_cpu.set_core_reg(CoreRegister::X0, *dtb_start)?;
                boot_cpu.set_core_reg(CoreRegister::X1, 0)?;
                boot_cpu.set_core_reg(CoreRegister::X2, 0)?;
                boot_cpu.set_core_reg(CoreRegister::X3, 0)?;
                boot_cpu.set_core_reg(CoreRegister::PC, *kernel_start)?;
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
}

impl<M, V> BootLoader<M, V> for AArch64BootLoader
where
    M: MemoryContainer,
    V: AArch64Vcpu,
{
    fn load(
        &self,
        ram_base: u64,
        memory: &mut MemoryAddressSpace<M>,
        vcpus: &mut Vec<V>,
    ) -> Result<()> {
        let mut layout = AArch64Layout::new(ram_base);

        self.load_kernel(&mut layout, memory)?;
        self.load_dtb(&mut layout, memory)?;
        self.setup_boot_cpu(&mut layout, vcpus)?;

        layout.finalize()?;

        Ok(())
    }
}
