use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::slice;

use tracing::debug;
use vm_firmware::x86_64::gdt::Gdt;
use vm_firmware::x86_64::gdt::GdtEntry;
use vm_mm::manager::MemoryAddressSpace;
use zerocopy::FromZeros;
use zerocopy::IntoBytes;

use crate::kernel_loader::Error;
use crate::kernel_loader::KernelLoader;
use crate::kernel_loader::LoadResult;
use crate::kernel_loader::Result;
use crate::kernel_loader::linux::bzimage::boot_params::BootE820Entry;
use crate::kernel_loader::linux::bzimage::boot_params::BootParams;
use crate::kernel_loader::linux::bzimage::boot_params::E820Type;

mod boot_params;

const MINIMAL_VERSION: u16 = 0x206;

fn to_gpa(cs: u16, ip: u16) -> u32 {
    ((cs as u32) << 4) + ip as u32
}

pub struct BzImageBootParams {
    pub gdt_start: u32,
    pub boot_params_start: u32,
    pub kernel_start: u32,
    // pub heap_end: u32,
    pub initrd_start: u32,
    pub cmdline_start: u32,
    pub memory_size: u64,
    pub mmio_start: u64,
    pub mmio_length: u64,
}

pub struct BzImage {
    bzimage: Vec<u8>,
    initrd: Option<Vec<u8>>,
    cmdline: Option<String>,
}

impl BzImage {
    pub fn new(path: &Path, initrd: Option<&Path>, cmdline: Option<&str>) -> Result<Self> {
        let bzimage = fs::read(path).map_err(|_| Error::ReadFailed)?;

        let initrd = initrd
            .map(fs::read)
            .transpose()
            .map_err(|_| Error::ReadFailed)?;
        let cmdline = cmdline.map(|s| s.to_string());

        Ok(BzImage {
            bzimage,
            initrd,
            cmdline,
        })
    }

    fn setup_hdr(
        &self,
        params: &BzImageBootParams,
        boot_params: &mut BootParams,
        memory: &MemoryAddressSpace,
    ) -> Result<LoadResult> {
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
            return Err(Error::InvalidKernelImage);
        }

        if boot_params.hdr.header != u32::from_le_bytes("HdrS".as_bytes().try_into().unwrap()) {
            return Err(Error::KernelTooOld);
        }

        let version = boot_params.hdr.version;
        if version < MINIMAL_VERSION {
            return Err(Error::KernelTooOld);
        }

        let kernel_version = {
            let kernel_version_offset = (boot_params.hdr.kernel_version + 0x200) as usize;
            let end = self.bzimage[kernel_version_offset..]
                .iter()
                .position(|&b| b == 0)
                .ok_or(Error::InvalidKernelImage)?;
            std::str::from_utf8(&self.bzimage[kernel_version_offset..kernel_version_offset + end])
                .map_err(|_| Error::InvalidKernelImage)?
        };
        debug!(kernel_version);

        boot_params.hdr.type_of_loader = 0xff; // undefined

        if boot_params.hdr.loadflags & 0x01 != 1 {
            // the proteced-mode code is loaded at 0x10000 which is not expected
            return Err(Error::InvalidKernelImage);
        }
        // boot_params.hdr.loadflags |= 0x80; // CAN_USE_HEAP

        // boot_params.hdr.heap_end_ptr =

        if params.kernel_start != boot_params.hdr.code32_start {
            return Err(Error::KernelStartOffsetNotSupport);
        }
        // We are booting using the 32-bit boot protocol
        // If booting 64-bit linux, we should plus 0x200 offset.
        let kernel_start = boot_params.hdr.code32_start as u64;

        if let Some(initrd) = &self.initrd {
            if params.initrd_start as usize + initrd.len()
                > boot_params.hdr.initrd_addr_max as usize
            {
                return Err(Error::InitramfsAddressTooHigh);
            }

            boot_params.hdr.ramdisk_image = params.initrd_start;
            boot_params.hdr.ramdisk_size = initrd
                .len()
                .try_into()
                .map_err(|_| Error::InitramfsTooLarge)?;
            memory
                .copy_from_slice(params.initrd_start as u64, initrd)
                .map_err(Error::CopyInitramfsFailed)?;
        }

        {
            let cmdline = if let Some(cmdline) = &self.cmdline {
                CString::new(cmdline.to_string()).map_err(|_| Error::CopyCmdlineFailed)?
            } else {
                CString::new("auto".to_string()).unwrap()
            };

            if cmdline.count_bytes() > boot_params.hdr.cmdline_size as usize {
                return Err(Error::CmdlineTooLarge);
            }

            boot_params.hdr.cmd_line_ptr = params.cmdline_start;

            memory
                .copy_from_slice(params.cmdline_start as u64, cmdline.as_bytes_with_nul())
                .map_err(|_| Error::CopyCmdlineFailed)?;
        }

        let kernel_len;
        {
            let mut setup_sects = boot_params.hdr.setup_sects;
            if setup_sects == 0 {
                setup_sects = 4;
            }

            let setup_size = (setup_sects as usize + 1) * 0x200;
            kernel_len = self.bzimage.len() - setup_size;
            memory
                .copy_from_slice(params.kernel_start as u64, &self.bzimage[setup_size..])
                .map_err(Error::CopyKernelFailed)?;
        }

        memory
            .copy_from_slice(params.boot_params_start as u64, unsafe {
                slice::from_raw_parts(
                    boot_params as *const BootParams as *const u8,
                    size_of::<BootParams>(),
                )
            })
            .map_err(Error::CopyKernelFailed)?;

        Ok(LoadResult {
            start_pc: kernel_start,
            kernel_start,
            kernel_len,
            gdt: Gdt::default(), // TODO: refine
        })
    }

    fn setup_e820(
        &self,
        mm: &MemoryAddressSpace,
        params: &BzImageBootParams,
        boot_params: &mut BootParams,
    ) -> Result<()> {
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
            addr: params.mmio_start,
            size: params.mmio_length,
            ty: E820Type::Reserved as u32,
        };
        index += 1;

        boot_params.e820_entries = index as u8;

        Ok(())
    }

    fn setup_gdt(&self, params: &BzImageBootParams, memory: &MemoryAddressSpace) -> Result<Gdt<5>> {
        let null = GdtEntry::new(0, 0, 0);
        let null2 = GdtEntry::new(0, 0, 0);
        let code = GdtEntry::new(0, 0xfffff, 0xc09b);
        let data = GdtEntry::new(0, 0xfffff, 0xc093);
        let tss = GdtEntry::new(0, 0xfffff, 0x808b);

        let gdt = Gdt::new([null, null2, code, data, tss]);

        memory
            .copy_from_slice(params.gdt_start as u64, gdt.as_bytes())
            .map_err(|_| Error::CopyGdtFailed)?;

        Ok(gdt)
    }
}

impl KernelLoader for BzImage {
    type BootParams = BzImageBootParams;

    fn load(
        &mut self,
        params: &Self::BootParams,
        memory: &MemoryAddressSpace,
    ) -> Result<LoadResult> {
        let mut boot_params = BootParams::new_zeroed();

        self.setup_e820(memory, params, &mut boot_params)?;
        let load_result = self.setup_hdr(params, &mut boot_params, memory)?;
        let gdt = self.setup_gdt(params, memory)?;

        if false {
            todo!("setup acpi_rsdp_addr");
        }

        Ok(LoadResult {
            start_pc: load_result.start_pc,
            kernel_start: load_result.kernel_start,
            kernel_len: load_result.kernel_len,
            gdt,
        })
    }
}
