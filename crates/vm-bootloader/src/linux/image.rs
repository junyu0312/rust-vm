/*
 * https://www.kernel.org/doc/html/v5.3/arm64/booting.html
 */

use std::fs;
use std::path::Path;

use anyhow::anyhow;
use anyhow::ensure;
use tracing::debug;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::vcpu::arch::aarch64::AArch64Vcpu;
use vm_core::vcpu::arch::aarch64::reg::CoreRegister;
use vm_core::vcpu::arch::aarch64::reg::cnthctl_el2::CnthctlEl2;
use vm_core::vcpu::arch::aarch64::reg::sctlr_el1::SctlrEl1;
use vm_core::virt::Virt;
use zerocopy::FromBytes;

use crate::BootLoader;
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
    pub fn new(
        kernel: &Path,
        initrd: Option<&Path>,
        cmdline: Option<&str>,
    ) -> anyhow::Result<Self> {
        let image = fs::read(kernel)?;
        let initrd = initrd.map(fs::read).transpose()?;
        let cmdline = cmdline.map(|s| s.to_string());

        Ok(Image {
            image,
            initrd,
            cmdline,
        })
    }

    fn get_header(&self) -> anyhow::Result<Header> {
        let len = size_of::<Header>();

        let header = Header::read_from_bytes(&self.image[0..len])
            .map_err(|_| anyhow!("Invalid image header"))?;

        debug!(?header);

        Ok(header)
    }

    fn validate(&self) -> anyhow::Result<()> {
        let len = size_of::<Header>();

        ensure!(self.image.len() >= len, "image too small");

        let header = self.get_header()?;

        ensure!(header.magic == 0x644d5241, "Invalid header magic");

        Ok(())
    }

    fn setup_memory<C>(&self, memory: &mut MemoryAddressSpace<C>) -> anyhow::Result<()>
    where
        C: MemoryContainer,
    {
        let header = self.get_header()?;

        ensure!(header.image_size != 0, "Image version too old");
        ensure!(header.text_offset == 0, "text_offset should be 0");

        let text_offset = DEFAULT_TEXT_OFFSET;

        memory.copy_from_slice(text_offset, &self.image, self.image.len())?;

        if let Some(_initrd) = &self.initrd {
            todo!()
        }

        if let Some(_cmdline) = &self.cmdline {
            todo!()
        }

        Ok(())
    }

    fn setup_device_tree(&self) -> anyhow::Result<()> {
        todo!()
    }

    #[allow(warnings)]
    fn setup_primary_cpu<V>(&self, primary_cpu: &mut V) -> anyhow::Result<()>
    where
        V: AArch64Vcpu,
    {
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
        memory: &mut MemoryAddressSpace<V::Memory>,
        _memory_size: usize,
        primary_cpu: &mut V::Vcpu,
    ) -> anyhow::Result<()> {
        self.validate()?;

        self.setup_memory(memory)?;

        self.setup_device_tree()?;

        self.setup_primary_cpu(primary_cpu)?;

        Ok(())
    }
}
