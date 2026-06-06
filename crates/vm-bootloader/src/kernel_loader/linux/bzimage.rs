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

    /*
    fn install(
        &self,
        _ram_base: u64,
        memory: &mut MemoryAddressSpace<V::Memory>,
        memory_size: usize,
        vcpu0: &mut V::Vcpu,
    ) -> Result<(), Error> {




        {
            let setup_start_gpa = to_gpa(CS, IP) as u64;

            // boot sector + setup code
            let setup_sects_size = (setup_sects + 1) as usize * 512;

            {
                // copy setup
                memory
                    .copy_from_slice(setup_start_gpa, &self.bzimage[..], setup_sects_size)
                    .map_err(|err| Error::CopyKernelFailed(err.to_string()))?;
            }

            {
                // copy kernel
                let buf = &self.bzimage[setup_sects_size..];
                memory
                    .copy_from_slice(KERNEL_START as u64, buf, buf.len())
                    .map_err(|err| Error::CopyKernelFailed(err.to_string()))?;
            }

            {
                // copy cmdline
                if let Some(cmdline) = &self.cmdline {
                    let cmdline_size = self.get_cmdline_size()? as usize;
                    let cstr =
                        CString::new(cmdline.to_string()).map_err(|_| Error::CopyCmdlineFailed)?;

                    let len = cstr.count_bytes();
                    if len >= cmdline_size {
                        return Err(Error::CopyCmdlineFailed);
                    }

                    memory
                        .memset(CMDLINE_OFFSET as u64, 0, cmdline_size)
                        .map_err(|_| Error::CopyCmdlineFailed)?;
                    memory
                        .copy_from_slice(
                            CMDLINE_OFFSET as u64,
                            cstr.as_bytes_with_nul(),
                            cstr.count_bytes(),
                        )
                        .map_err(|_| Error::CopyCmdlineFailed)?;
                }
            }

            {
                // copy initramfs
                if let Some(initrd) = &self.initrd {
                    let initrd_address = memory_size.min(
                        self.get_initrd_addr_max()
                            .map_err(|_| Error::SetupInitrdFailed)?
                            as usize,
                    );
                    let initrd_address = initrd_address as u32 - initrd.len() as u32;

                    memory
                        .copy_from_slice(initrd_address as u64, initrd, initrd.len())
                        .map_err(|_| Error::SetupInitrdFailed)?;

                    unsafe {
                        let ptr = memory
                            .gpa_to_hva(setup_start_gpa + RAMDISK_IMAGE.offset as u64)
                            .map_err(|_| Error::SetupInitrdFailed)?
                            as *mut u32;
                        *ptr = initrd_address;
                    }

                    unsafe {
                        let ptr = memory
                            .gpa_to_hva(setup_start_gpa + RAMDISK_SIZE.offset as u64)
                            .map_err(|_| Error::SetupInitrdFailed)?
                            as *mut u32;
                        *ptr = initrd.len() as u32;
                    }
                }
            }

            unsafe {
                let ptr = memory
                    .gpa_to_hva(setup_start_gpa + CMD_LINE_PTR.offset as u64)
                    .map_err(|_| Error::SetupKernelFailed)? as *mut u32;
                *ptr = CMDLINE_OFFSET;
            }
            unsafe {
                let ptr = memory
                    .gpa_to_hva(setup_start_gpa + HEAP_END_PTR.offset as u64)
                    .map_err(|_| Error::SetupKernelFailed)? as *mut u16;
                *ptr = 0xfe00;
            }
            unsafe {
                let ptr = memory
                    .gpa_to_hva(setup_start_gpa + TYPE_OF_LOADER.offset as u64)
                    .map_err(|_| Error::SetupKernelFailed)?;
                *ptr = 0xff; // undefined
            }
            unsafe {
                let ptr = memory
                    .gpa_to_hva(setup_start_gpa + LOADFLAGS.offset as u64)
                    .map_err(|_| Error::SetupKernelFailed)?;
                *ptr |= 0x80;
            }

            {
                // To meet kvmtool bios
                {
                    const VGA_ROM_BEGIN: u64 = 0x000c0000;
                    const VGA_ROM_OEM_STRING: u64 = VGA_ROM_BEGIN;
                    const VGA_ROM_OEM_STRING_SIZE: usize = 16;
                    const VGA_ROM_MODES: u64 = VGA_ROM_OEM_STRING + VGA_ROM_OEM_STRING_SIZE as u64;

                    memory
                        .copy_from_slice(
                            VGA_ROM_BEGIN,
                            &[0; VGA_ROM_OEM_STRING_SIZE],
                            VGA_ROM_OEM_STRING_SIZE,
                        )
                        .map_err(|_| Error::SetupFirmwareFailed)?;
                    let s =
                        CString::from_str("KVM VESA").map_err(|_| Error::SetupFirmwareFailed)?;
                    memory
                        .copy_from_slice(VGA_ROM_BEGIN, s.as_bytes(), s.count_bytes())
                        .map_err(|_| Error::SetupFirmwareFailed)?;

                    memory
                        .copy_from_slice(VGA_ROM_MODES, &0x0112u16.to_le_bytes(), 2)
                        .map_err(|_| Error::SetupFirmwareFailed)?;
                    memory
                        .copy_from_slice(VGA_ROM_MODES + 2, &0x0ffffu16.to_le_bytes(), 2)
                        .map_err(|_| Error::SetupFirmwareFailed)?;
                }
            }
        }

        {
            let mut regs = vcpu0.get_regs().map_err(|_| Error::SetupBootcpuFailed)?;
            regs.rip = IP as u64 + 0x200;
            regs.rsp = SP as u64;
            regs.rbp = SP as u64;
            regs.rflags = 0x2;
            vcpu0
                .set_regs(&regs)
                .map_err(|_| Error::SetupBootcpuFailed)?;

            let mut sregs = vcpu0.get_sregs().map_err(|_| Error::SetupBootcpuFailed)?;
            sregs.cs.selector = CS;
            sregs.cs.base = (CS as u64) << 4;
            sregs.ss.selector = CS;
            sregs.ss.base = (CS as u64) << 4;
            sregs.ds.selector = CS;
            sregs.ds.base = (CS as u64) << 4;
            sregs.fs.selector = CS;
            sregs.fs.base = (CS as u64) << 4;
            sregs.gs.selector = CS;
            sregs.gs.base = (CS as u64) << 4;
            vcpu0
                .set_sregs(&sregs)
                .map_err(|_| Error::SetupBootcpuFailed)?;
        }

        Ok(())
    }
     */
}
