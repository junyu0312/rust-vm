use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::slice;

use tracing::debug;
use vm_firmware::acpi::builder::AcpiTableBuilder;
use vm_firmware::x86_64::gdt::Gdt;
use vm_firmware::x86_64::gdt::GdtEntry;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::FromZeros;
use zerocopy::IntoBytes;

use crate::initrd_loader::InitrdLoadResult;
use crate::kernel_loader::error::KernelLoaderError;
use crate::kernel_loader::linux::bzimage::boot_params::BootE820Entry;
use crate::kernel_loader::linux::bzimage::boot_params::BootParams;
use crate::kernel_loader::linux::bzimage::boot_params::E820Type;

mod boot_params;

const MINIMAL_VERSION: u16 = 0x206;

pub struct LoadResult {
    pub start_pc: u32,
    pub gdt: Gdt<5>,
}

pub struct BzImageBootParams {
    pub vcpus: usize,
    pub definition_block: Vec<u8>,
    pub gdt_start: u32,
    pub boot_params_start: u32,
    // pub heap_end: u32,
    pub cmdline_start: u32,
    pub acpi_rsdt_addr: u32,
    pub acpi_max_length: u32,
    pub kernel_start: u32,
    pub initrd: Option<InitrdLoadResult>,
    pub mmio_start: u32,
    pub mmio_length: u32,
    pub ecam_base: u32,
    pub ecam_length: u32,
    pub ioapic_base_addr: u32,
    pub apic_base_addr: u32,
}

pub struct BzImage {
    bzimage: Vec<u8>,
    cmdline: Option<String>,
}

impl BzImage {
    pub fn new(path: &Path, cmdline: Option<&str>) -> Result<Self, KernelLoaderError> {
        let bzimage = fs::read(path).map_err(|_| KernelLoaderError::ReadFailed)?;
        let cmdline = cmdline.map(|s| s.to_string());

        Ok(BzImage { bzimage, cmdline })
    }

    pub fn load(
        &mut self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        params: &BzImageBootParams,
    ) -> Result<LoadResult, KernelLoaderError> {
        let mut boot_params = BootParams::new_zeroed();

        self.setup_acpi(ram_allocator, memory, params, &mut boot_params)?;
        self.setup_e820(memory, params, &mut boot_params)?;

        let start_pc = self.setup_hdr(ram_allocator, params, &mut boot_params, memory)?;
        let gdt = self.setup_gdt(ram_allocator, params, memory)?;

        Ok(LoadResult { start_pc, gdt })
    }

    fn setup_hdr(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        params: &BzImageBootParams,
        boot_params: &mut BootParams,
        memory: &MemoryAddressSpace,
    ) -> Result<u32, KernelLoaderError> {
        {
            // the second byte of `jump` field is a signed offset relative to byte 0x202,
            // which can be used to determine the size of the header
            let end_of_hdr = 0x0202 + self.bzimage[0x0201] as usize;
            let length_of_hdr = end_of_hdr - 0x01f1;
            boot_params.hdr.as_mut_bytes()[..length_of_hdr]
                .copy_from_slice(&self.bzimage[0x01f1..end_of_hdr]);
        }

        debug!(?boot_params.hdr);

        if boot_params.hdr.boot_flag != 0xAA55 {
            return Err(KernelLoaderError::InvalidKernelImage);
        }

        if boot_params.hdr.header != u32::from_le_bytes("HdrS".as_bytes().try_into().unwrap()) {
            return Err(KernelLoaderError::KernelTooOld);
        }

        let version = boot_params.hdr.version;
        if version < MINIMAL_VERSION {
            return Err(KernelLoaderError::KernelTooOld);
        }

        let kernel_version = {
            let kernel_version_offset = (boot_params.hdr.kernel_version + 0x200) as usize;
            let end = self.bzimage[kernel_version_offset..]
                .iter()
                .position(|&b| b == 0)
                .ok_or(KernelLoaderError::InvalidKernelImage)?;
            std::str::from_utf8(&self.bzimage[kernel_version_offset..kernel_version_offset + end])
                .map_err(|_| KernelLoaderError::InvalidKernelImage)?
        };
        debug!(kernel_version);

        boot_params.hdr.type_of_loader = 0xff; // undefined

        if boot_params.hdr.loadflags & 0x01 != 1 {
            // the proteced-mode code is loaded at 0x10000 which is not expected
            return Err(KernelLoaderError::InvalidKernelImage);
        }
        // boot_params.hdr.loadflags |= 0x80; // CAN_USE_HEAP

        // boot_params.hdr.heap_end_ptr =

        if params.kernel_start != boot_params.hdr.code32_start {
            return Err(KernelLoaderError::KernelStartOffsetNotSupport);
        }
        // We are booting using the 32-bit boot protocol
        // If booting 64-bit linux, we should plus 0x200 offset.
        let kernel_start = boot_params.hdr.code32_start;

        if let Some(initrd) = &params.initrd {
            if initrd.initrd_start + initrd.initrd_len as u64
                > boot_params.hdr.initrd_addr_max as u64
            {
                return Err(KernelLoaderError::InitramfsAddressTooHigh);
            }

            boot_params.hdr.ramdisk_image = initrd
                .initrd_start
                .try_into()
                .map_err(|_| KernelLoaderError::InitramfsAddressTooHigh)?;
            boot_params.hdr.ramdisk_size = initrd
                .initrd_len
                .try_into()
                .map_err(|_| KernelLoaderError::InitramfsTooLarge)?;
        }

        {
            let cmdline = if let Some(cmdline) = &self.cmdline {
                CString::new(cmdline.to_string())
                    .map_err(|_| KernelLoaderError::CopyCmdlineFailed)?
            } else {
                CString::new("auto".to_string()).unwrap()
            };

            if cmdline.count_bytes() > boot_params.hdr.cmdline_size as usize {
                return Err(KernelLoaderError::CmdlineTooLarge);
            }

            boot_params.hdr.cmd_line_ptr = params.cmdline_start;

            let buf = cmdline.as_bytes_with_nul();
            let range = ram_allocator.reserve(params.cmdline_start as u64, buf.len())?;
            memory
                .copy_from_slice(range.start, cmdline.as_bytes_with_nul())
                .map_err(|_| KernelLoaderError::CopyCmdlineFailed)?;
        }

        let kernel_len;
        {
            let mut setup_sects = boot_params.hdr.setup_sects;
            if setup_sects == 0 {
                setup_sects = 4;
            }

            let setup_size = (setup_sects as usize + 1) * 0x200;
            kernel_len = self.bzimage.len() - setup_size;
            let range = ram_allocator.reserve(params.kernel_start as u64, kernel_len)?;
            memory
                .copy_from_slice(range.start, &self.bzimage[setup_size..])
                .map_err(KernelLoaderError::CopyKernelFailed)?;
        }

        {
            let range =
                ram_allocator.reserve(params.boot_params_start as u64, size_of::<BootParams>())?;
            memory
                .copy_from_slice(range.start, unsafe {
                    slice::from_raw_parts(
                        boot_params as *const BootParams as *const u8,
                        size_of::<BootParams>(),
                    )
                })
                .map_err(KernelLoaderError::CopyKernelFailed)?;
        };

        Ok(kernel_start)
    }

    fn setup_acpi(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        mm: &MemoryAddressSpace,
        params: &BzImageBootParams,
        boot_params: &mut BootParams,
    ) -> Result<(), KernelLoaderError> {
        let _ = ram_allocator.reserve(
            params.acpi_rsdt_addr as u64,
            params.acpi_max_length as usize,
        )?;

        let mut acpi_ram_allocator = RangeAllocator::<u64>::default();
        acpi_ram_allocator.insert(
            params.acpi_rsdt_addr as u64,
            params.acpi_max_length as usize,
        )?;

        let acpi = AcpiTableBuilder::default()
            .set_vcpus(
                params
                    .vcpus
                    .try_into()
                    .map_err(|_| KernelLoaderError::VcpuExceedsAcpiCapability)?,
            )?
            .set_definition_block(params.definition_block.clone())?
            .set_apic_base_address(params.apic_base_addr)?
            .set_io_apic_address(params.ioapic_base_addr)?
            .set_pci_mmio_base_addr(params.ecam_base as u64)?
            .build()?;

        acpi.install(&mut acpi_ram_allocator, mm, params.acpi_rsdt_addr as u64)?;
        boot_params.acpi_rsdp_addr = params.acpi_rsdt_addr as u64;

        Ok(())
    }

    fn setup_e820(
        &self,
        mm: &MemoryAddressSpace,
        params: &BzImageBootParams,
        boot_params: &mut BootParams,
    ) -> Result<(), KernelLoaderError> {
        let mut index = 0;

        for region in mm.regions().values() {
            boot_params.e820_table[index] = BootE820Entry {
                addr: region.gpa,
                size: region.len() as u64,
                ty: E820Type::Ram as u32,
            };
            index += 1;
        }

        boot_params.e820_table[index] = BootE820Entry {
            addr: params.acpi_rsdt_addr as u64,
            size: params.acpi_max_length as u64,
            ty: E820Type::Acpi as u32,
        };
        index += 1;

        boot_params.e820_table[index] = BootE820Entry {
            addr: params.mmio_start as u64,
            size: params.mmio_length as u64,
            ty: E820Type::Reserved as u32,
        };
        index += 1;

        boot_params.e820_table[index] = BootE820Entry {
            addr: params.ecam_base as u64,
            size: params.ecam_length as u64,
            ty: E820Type::Reserved as u32,
        };
        index += 1;

        boot_params.e820_entries = index as u8;

        Ok(())
    }

    fn setup_gdt(
        &self,
        ram_allocator: &mut RangeAllocator<u64>,
        params: &BzImageBootParams,
        memory: &MemoryAddressSpace,
    ) -> Result<Gdt<5>, KernelLoaderError> {
        let null = GdtEntry::new(0, 0, 0);
        let null2 = GdtEntry::new(0, 0, 0);
        let code = GdtEntry::new(0, 0xfffff, 0xc09b);
        let data = GdtEntry::new(0, 0xfffff, 0xc093);
        let tss = GdtEntry::new(0, 0xfffff, 0x808b);

        let gdt = Gdt::new([null, null2, code, data, tss]);

        ram_allocator.reserve(params.gdt_start as u64, gdt.as_bytes().len())?;
        memory
            .copy_from_slice(params.gdt_start as u64, gdt.as_bytes())
            .map_err(|_| KernelLoaderError::CopyGdtFailed)?;

        Ok(gdt)
    }
}
