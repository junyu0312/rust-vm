/*
 * https://www.kernel.org/doc/html/v5.3/arm64/booting.html
 */

use std::fs;
use std::path::Path;

use tracing::debug;
use vm_mm::manager::MemoryAddressSpace;
use vm_utils::range_allocator::RangeAllocator;
use zerocopy::FromBytes;

use crate::kernel_loader::error::KernelLoaderError;

const DEFAULT_TEXT_OFFSET: u64 = 0x80000;

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

pub struct AArch64BootParams {
    pub ram_base: u64,
}

pub struct LoadResult {
    pub start_pc: u64,
}

pub struct Image {
    kernel: Vec<u8>,
}

impl Image {
    pub fn new(kernel: &Path) -> Result<Self, KernelLoaderError> {
        let kernel = fs::read(kernel).map_err(|_| KernelLoaderError::ReadFailed)?;

        let image = Image { kernel };

        image.validate()?;

        Ok(image)
    }

    pub fn load(
        &mut self,
        ram_allocator: &mut RangeAllocator<u64>,
        memory: &MemoryAddressSpace,
        boot_params: &AArch64BootParams,
    ) -> Result<LoadResult, KernelLoaderError> {
        let header = self.get_header()?;

        let text_offset = if header.image_size == 0 {
            DEFAULT_TEXT_OFFSET
        } else {
            header.text_offset
        };

        let kernel_len = if header.image_size == 0 {
            self.kernel.len()
        } else {
            header
                .image_size
                .try_into()
                .map_err(|_| KernelLoaderError::InvalidKernelImage)?
        };

        // Check 2MB alignment
        if !boot_params.ram_base.is_multiple_of(2 << 20) {
            return Err(KernelLoaderError::InvalidAddressAlignment);
        }

        let kernel_start = boot_params.ram_base + text_offset;

        ram_allocator.reserve(kernel_start, kernel_len)?;
        memory
            .copy_from_slice(kernel_start, &self.kernel)
            .map_err(KernelLoaderError::CopyKernelFailed)?;

        Ok(LoadResult {
            start_pc: kernel_start,
        })
    }

    fn get_header(&self) -> Result<Header, KernelLoaderError> {
        let len = size_of::<Header>();

        let header = Header::read_from_bytes(&self.kernel[0..len])
            .map_err(|_| KernelLoaderError::InvalidKernelImage)?;

        debug!(?header);

        Ok(header)
    }

    fn validate(&self) -> Result<(), KernelLoaderError> {
        let len = size_of::<Header>();

        if self.kernel.len() < len {
            return Err(KernelLoaderError::InvalidKernelImage);
        }

        let header = self.get_header()?;

        if header.magic != 0x644d5241 {
            return Err(KernelLoaderError::InvalidKernelImage);
        }

        Ok(())
    }
}
