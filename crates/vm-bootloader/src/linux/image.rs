/*
 * https://www.kernel.org/doc/html/v5.3/arm64/booting.html
 */

use std::fs;
use std::path::Path;

use tracing::debug;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::utils::fdt::DtbBuilder;
use vm_core::vcpu::arch::aarch64::AArch64Vcpu;
use vm_core::vcpu::arch::aarch64::reg::CoreRegister;
use vm_core::vcpu::arch::aarch64::reg::cnthctl_el2::CnthctlEl2;
use vm_core::vcpu::arch::aarch64::reg::sctlr_el1::SctlrEl1;
use vm_core::virt::Virt;
use zerocopy::FromBytes;

use crate::BootLoader;
use crate::Error;
use crate::linux::image::layout::DEFAULT_TEXT_OFFSET;

mod layout {
    pub const DEFAULT_TEXT_OFFSET: u64 = 0x80000;
}

#[repr(C)]
#[derive(Debug, FromBytes)]
struct Header {
    code0: u32,
    code1: u32,
    text_offset: u64,
    image_size: u64,
    flags: u64,
    res2: u64,
    res3: u64,
    res4: u64,
    magic: u32,
    res5: u32,
}

#[allow(dead_code)]
pub struct Image {
    image: Vec<u8>,
    initrd: Option<Vec<u8>>,
    cmdline: Option<String>,
}

impl Image {
    pub fn new(kernel: &Path, initrd: Option<&Path>, cmdline: Option<&str>) -> Result<Self, Error> {
        let image = fs::read(kernel).map_err(|_| Error::ReadFailed)?;
        let initrd = initrd
            .map(fs::read)
            .transpose()
            .map_err(|_| Error::ReadFailed)?;
        let cmdline = cmdline.map(|s| s.to_string());

        Ok(Image {
            image,
            initrd,
            cmdline,
        })
    }

    fn get_header(&self) -> Result<Header, Error> {
        let len = size_of::<Header>();

        let header =
            Header::read_from_bytes(&self.image[0..len]).map_err(|_| Error::InvalidKernelImage)?;

        debug!(?header);

        Ok(header)
    }

    fn validate(&self) -> Result<(), Error> {
        let len = size_of::<Header>();

        if self.image.len() < len {
            return Err(Error::InvalidKernelImage);
        }

        let header = self.get_header()?;

        if header.magic != 0x644d5241 {
            return Err(Error::InvalidKernelImage);
        }

        Ok(())
    }

    fn setup_memory<C>(
        &self,
        ram_base: u64,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<(), Error>
    where
        C: MemoryContainer,
    {
        let header = self.get_header()?;

        let text_offset = if header.image_size == 0 {
            DEFAULT_TEXT_OFFSET
        } else {
            header.text_offset
        };

        // check 2M align
        if !ram_base.is_multiple_of(2 << 20) {
            return Err(Error::InvalidAddressAlignment);
        }

        let offset = ram_base + text_offset;

        memory
            .copy_from_slice(offset, &self.image, self.image.len())
            .map_err(|err| Error::CopyKernelFailed(err.to_string()))?;

        if let Some(_initrd) = &self.initrd {
            todo!()
        }

        if let Some(_cmdline) = &self.cmdline {
            todo!()
        }

        Ok(())
    }

    fn setup_device_tree(&self) -> Result<(), Error> {
        let _fdt = DtbBuilder::build_dtb().map_err(|err| Error::SetupDtbFailed(err.to_string()))?;

        todo!();
    }

    #[allow(warnings)]
    fn setup_primary_cpu<V>(&self, primary_cpu: &mut V) -> Result<(), Error>
    where
        V: AArch64Vcpu,
    {
        let setup = || {
            {
                // Setup general-purpose register

                let gpa_of_dtb = todo!();
                primary_cpu.set_core_reg(CoreRegister::X0, gpa_of_dtb)?;
                primary_cpu.set_core_reg(CoreRegister::X1, 0)?;
                primary_cpu.set_core_reg(CoreRegister::X2, 0)?;
                primary_cpu.set_core_reg(CoreRegister::X3, 0)?;
            }

            {
                // CPU mode

                let mut pstate = primary_cpu.get_core_reg(CoreRegister::PState)?;
                pstate |= 0x03C0; // DAIF
                pstate &= !0xf; // Clear low 4 bits
                pstate |= 0x0005; // El1h
                primary_cpu.set_core_reg(CoreRegister::PState, pstate)?;

                // more, non secure el1
                todo!()
            }

            {
                // Caches, MMUs

                let sctlr_el1 = primary_cpu.get_sctlr_el1()?;
                sctlr_el1.remove(SctlrEl1::M); // Disable MMU
                sctlr_el1.remove(SctlrEl1::I); // Disable I-cache
                primary_cpu.set_sctlr_el1(sctlr_el1)?;
            }

            {
                // Architected timers

                todo!(
                    "CNTFRQ must be programmed with the timer frequency and CNTVOFF must be programmed with a consistent value on all CPUs."
                );

                let cnthctl_el2 = primary_cpu.get_cnthctl_el2()?;
                cnthctl_el2.insert(CnthctlEl2::EL1PCTEN); // TODO: or bit0?(https://www.kernel.org/doc/html/v5.3/arm64/booting.html)
                primary_cpu.set_cnthctl_el2(cnthctl_el2)?;
            }

            {
                // Coherency

                // Do nothing
            }

            {
                // System registers

                todo!()
            }

            anyhow::Ok(())
        };

        setup().map_err(|_| Error::SetupBootcpuFailed)?;

        Ok(())
    }
}

impl<V> BootLoader<V> for Image
where
    V: Virt,
    V::Vcpu: AArch64Vcpu,
{
    fn install(
        &self,
        ram_base: u64,
        memory: &mut MemoryAddressSpace<V::Memory>,
        _memory_size: usize,
        primary_cpu: &mut V::Vcpu,
    ) -> Result<(), Error> {
        self.validate()?;

        self.setup_memory(ram_base, memory)?;

        self.setup_device_tree()?;

        self.setup_primary_cpu(primary_cpu)?;

        Ok(())
    }
}
