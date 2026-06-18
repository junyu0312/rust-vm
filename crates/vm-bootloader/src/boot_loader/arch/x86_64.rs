use std::ffi::CString;
use std::path::Path;
use std::path::PathBuf;
use std::slice::Iter;

use async_trait::async_trait;
use vm_core::arch::irq::InterruptController;
use vm_core::arch::x86_64::layout::ACPI_MAX_LEN;
use vm_core::arch::x86_64::layout::ACPI_RSDT_START;
use vm_core::arch::x86_64::layout::APIC_ADDR;
use vm_core::arch::x86_64::layout::BOOT_PARAMS_START;
use vm_core::arch::x86_64::layout::CMDLINE_START;
use vm_core::arch::x86_64::layout::ECAM_BASE;
use vm_core::arch::x86_64::layout::ECAM_LENGTH;
use vm_core::arch::x86_64::layout::GDT_START;
use vm_core::arch::x86_64::layout::INITRD_START;
use vm_core::arch::x86_64::layout::IOAPIC_ADDR;
use vm_core::arch::x86_64::layout::KERNEL_START;
use vm_core::arch::x86_64::layout::MMIO_LEN;
use vm_core::arch::x86_64::layout::MMIO_START;
use vm_core::cpu::vcpu::Vcpu;
use vm_core::device::Device;
use vm_firmware::acpi::builder::AcpiTableBuilder;
use vm_firmware::x86_64::gdt::Gdt;
use vm_firmware::x86_64::gdt::GdtEntry;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::IntoBytes;

use crate::boot_loader::BootLoader;
use crate::boot_loader::BootLoaderBuilder;
use crate::boot_loader::Error;
use crate::boot_loader::Result;
use crate::initrd_loader::InitrdLoadResult;
use crate::initrd_loader::InitrdLoader;
use crate::kernel_loader::linux::x86_64::bzimage::BzImage;
use crate::kernel_loader::linux::x86_64::bzimage::BzImageBootParams;
use crate::kernel_loader::linux::x86_64::bzimage::LoadResult;
use crate::utils::aml::build_definition_block;

pub struct X86_64BootLoader {
    kernel: PathBuf,
    initramfs: Option<PathBuf>,
    cmdline: Option<String>,
}

impl X86_64BootLoader {
    fn setup_acpi(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        mm: &MemoryAddressSpace,
        vcpus: usize,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<(u32, u32)> {
        let acpi_rsdt_addr = ACPI_RSDT_START as u64;
        let acpi_max_length = ACPI_MAX_LEN as usize;

        let _ = ram_allocator.reserve(acpi_rsdt_addr, acpi_max_length)?;

        let mut acpi_ram_allocator = RangeAllocator::<u64>::default();
        acpi_ram_allocator.insert(acpi_rsdt_addr, acpi_max_length)?;

        let acpi = AcpiTableBuilder::default()
            .set_vcpus(
                vcpus
                    .try_into()
                    .map_err(|_| Error::VcpuExceedsAcpiCapability)?,
            )?
            .set_definition_block(build_definition_block(devices))?
            .set_apic_base_address(APIC_ADDR)?
            .set_io_apic_address(IOAPIC_ADDR)?
            .set_pci_mmio_base_addr(ECAM_BASE as u64)?
            .build()?;

        acpi.install(&mut acpi_ram_allocator, mm, acpi_rsdt_addr)?;

        Ok((
            acpi_rsdt_addr.try_into().unwrap(),
            acpi_max_length.try_into().unwrap(),
        ))
    }

    fn setup_gdt(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
    ) -> Result<(Gdt<5>, u32)> {
        let gdt_start = GDT_START as u64;

        let null = GdtEntry::new(0, 0, 0);
        let null2 = GdtEntry::new(0, 0, 0);
        let code = GdtEntry::new(0, 0xfffff, 0xc09b);
        let data = GdtEntry::new(0, 0xfffff, 0xc093);
        let tss = GdtEntry::new(0, 0xfffff, 0x808b);

        let gdt = Gdt::new([null, null2, code, data, tss]);

        ram_allocator.reserve(gdt_start, gdt.as_bytes().len())?;
        memory
            .copy_from_slice(gdt_start, gdt.as_bytes())
            .map_err(|_| Error::Gdt("Failed to copy gdt".to_string()))?;

        Ok((gdt, gdt_start.try_into().unwrap()))
    }

    fn load_initramfs(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        initramfs: &Path,
    ) -> Result<InitrdLoadResult> {
        let result =
            InitrdLoader::new(initramfs)?.load(ram_allocator, memory, INITRD_START as u64)?;

        Ok(result)
    }

    fn load_cmdline(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
    ) -> Result<(u32, u32)> {
        let cmdline_start = CMDLINE_START;

        let cmdline = if let Some(cmdline) = &self.cmdline {
            CString::new(cmdline.to_string()).map_err(|err| Error::Cmdline(err.to_string()))?
        } else {
            CString::new("auto".to_string()).unwrap()
        };

        let buf = cmdline.as_bytes_with_nul();
        let range = ram_allocator.reserve(cmdline_start as u64, buf.len())?;
        memory
            .copy_from_slice(range.start, cmdline.as_bytes_with_nul())
            .map_err(|err| Error::Cmdline(err.to_string()))?;

        Ok((cmdline_start, buf.len() as u32))
    }

    fn load_image(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        acpi_rsdt_addr: u32,
        acpi_max_length: u32,
        initrd_load_result: Option<InitrdLoadResult>,
        cmdline_start: u32,
        cmdline_len: u32,
    ) -> Result<LoadResult> {
        let params = BzImageBootParams {
            boot_params_start: BOOT_PARAMS_START,
            cmdline_start,
            cmdline_len,
            acpi_rsdt_addr,
            acpi_max_length,
            kernel_start: KERNEL_START,
            initrd: initrd_load_result,
            mmio_start: MMIO_START,
            mmio_length: MMIO_LEN,
            ecam_base: ECAM_BASE,
            ecam_length: ECAM_LENGTH,
        };

        let load_result = BzImage::new(&self.kernel)?.load(ram_allocator, memory, &params)?;

        Ok(load_result)
    }
}

impl BootLoaderBuilder for X86_64BootLoader {
    fn new(kernel: PathBuf, initramfs: Option<PathBuf>, cmdline: Option<String>) -> Self {
        X86_64BootLoader {
            kernel,
            initramfs,
            cmdline,
        }
    }
}

#[async_trait]
impl BootLoader for X86_64BootLoader {
    async fn load(
        &self,
        _ram_size: u64,
        vcpus: usize,
        boot_vcpu: &mut Vcpu,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        _irq_chip: &dyn InterruptController,
        devices: Iter<'_, Box<dyn Device>>,
    ) -> Result<()> {
        let (acpi_rsdt_addr, acpi_max_length) =
            self.setup_acpi(ram_allocator, memory, vcpus, devices)?;

        let (gdt, gdt_start) = self.setup_gdt(ram_allocator, memory)?;

        let initrd_load_result = if let Some(initramfs) = &self.initramfs {
            Some(self.load_initramfs(ram_allocator, memory, initramfs)?)
        } else {
            None
        };

        let (cmdline_start, cmdline_len) = self.load_cmdline(ram_allocator, memory)?;

        let load_result = self.load_image(
            ram_allocator,
            memory,
            acpi_rsdt_addr,
            acpi_max_length,
            initrd_load_result,
            cmdline_start,
            cmdline_len,
        )?;

        boot_vcpu
            .setup_vcpu(
                gdt,
                gdt_start,
                load_result.boot_params_start,
                load_result.start_pc,
            )
            .await?;

        Ok(())
    }
}
