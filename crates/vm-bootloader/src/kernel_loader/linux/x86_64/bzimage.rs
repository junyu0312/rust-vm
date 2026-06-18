use std::fs;
use std::path::Path;

use tracing::debug;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::FromZeros;
use zerocopy::IntoBytes;

use crate::initrd_loader::InitrdLoadResult;
use crate::kernel_loader::error::KernelLoaderError;
use crate::kernel_loader::linux::x86_64::zero_page::SetupHeader;

const MINIMAL_VERSION: u16 = 0x206;

pub struct LoadResult {
    pub start_pc: u32,
    pub setup_hdr: SetupHeader,
}

pub struct BzImageBootParams {
    // pub heap_end: u32,
    pub cmdline_start: u32,
    pub cmdline_len: u32,
    pub kernel_start: u32,
    pub initrd: Option<InitrdLoadResult>,
}

pub struct BzImage {
    bzimage: Vec<u8>,
}

impl BzImage {
    pub fn new(path: &Path) -> Result<Self, KernelLoaderError> {
        let bzimage = fs::read(path).map_err(|_| KernelLoaderError::ReadFailed)?;

        Ok(BzImage { bzimage })
    }

    pub fn load(
        &mut self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        params: &BzImageBootParams,
    ) -> Result<LoadResult, KernelLoaderError> {
        let mut setup_hdr = SetupHeader::new_zeroed();

        {
            // the second byte of `jump` field is a signed offset relative to byte 0x202,
            // which can be used to determine the size of the header
            let end_of_hdr = 0x0202 + self.bzimage[0x0201] as usize;
            let length_of_hdr = end_of_hdr - 0x01f1;
            setup_hdr.as_mut_bytes()[..length_of_hdr]
                .copy_from_slice(&self.bzimage[0x01f1..end_of_hdr]);
        };

        debug!(?setup_hdr);

        if setup_hdr.boot_flag != 0xAA55 {
            return Err(KernelLoaderError::InvalidKernelImage);
        }

        if setup_hdr.header != u32::from_le_bytes("HdrS".as_bytes().try_into().unwrap()) {
            return Err(KernelLoaderError::KernelTooOld);
        }

        let version = setup_hdr.version;
        if version < MINIMAL_VERSION {
            return Err(KernelLoaderError::KernelTooOld);
        }

        let kernel_version = {
            let kernel_version_offset = (setup_hdr.kernel_version + 0x200) as usize;
            let end = self.bzimage[kernel_version_offset..]
                .iter()
                .position(|&b| b == 0)
                .ok_or(KernelLoaderError::InvalidKernelImage)?;
            std::str::from_utf8(&self.bzimage[kernel_version_offset..kernel_version_offset + end])
                .map_err(|_| KernelLoaderError::InvalidKernelImage)?
        };
        debug!(kernel_version);

        setup_hdr.type_of_loader = 0xff; // undefined

        if setup_hdr.loadflags & 0x01 != 1 {
            // the proteced-mode code is loaded at 0x10000 which is not expected
            return Err(KernelLoaderError::InvalidKernelImage);
        }
        // boot_params.hdr.loadflags |= 0x80; // CAN_USE_HEAP

        // boot_params.hdr.heap_end_ptr =

        if params.kernel_start != setup_hdr.code32_start {
            return Err(KernelLoaderError::KernelStartOffsetNotSupport);
        }
        // We are booting using the 32-bit boot protocol
        // If booting 64-bit linux, we should plus 0x200 offset.
        let kernel_start = setup_hdr.code32_start;

        if let Some(initrd) = &params.initrd {
            if initrd.initrd_start + initrd.initrd_len as u64 > setup_hdr.initrd_addr_max as u64 {
                return Err(KernelLoaderError::InitramfsAddressTooHigh);
            }

            setup_hdr.ramdisk_image = initrd
                .initrd_start
                .try_into()
                .map_err(|_| KernelLoaderError::InitramfsAddressTooHigh)?;
            setup_hdr.ramdisk_size = initrd
                .initrd_len
                .try_into()
                .map_err(|_| KernelLoaderError::InitramfsTooLarge)?;
        }

        {
            if params.cmdline_len > setup_hdr.cmdline_size {
                return Err(KernelLoaderError::CmdlineTooLarge);
            }

            setup_hdr.cmd_line_ptr = params.cmdline_start;
        }

        {
            let mut setup_sects = setup_hdr.setup_sects;
            if setup_sects == 0 {
                setup_sects = 4;
            }

            let setup_size = (setup_sects as usize + 1) * 0x200;
            let kernel_len = self.bzimage.len() - setup_size;
            let range = ram_allocator.reserve(params.kernel_start as u64, kernel_len)?;
            memory
                .copy_from_slice(range.start, &self.bzimage[setup_size..])
                .map_err(KernelLoaderError::CopyKernelFailed)?;
        }

        Ok(LoadResult {
            start_pc: kernel_start,
            setup_hdr,
        })
    }
}
