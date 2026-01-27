/*
 * https://www.kernel.org/doc/html/v5.3/arm64/booting.html
 */

use std::fs;
use std::path::Path;

use tracing::debug;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use zerocopy::FromBytes;

use crate::kernel_loader::Error;
use crate::kernel_loader::KernelLoader;
use crate::kernel_loader::LoadResult;
use crate::kernel_loader::Result;

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

pub struct Image {
    kernel: Vec<u8>,
}

impl Image {
    pub fn new(kernel: &Path) -> Result<Self> {
        let kernel = fs::read(kernel).map_err(|_| Error::ReadFailed)?;

        let image = Image { kernel };

        image.validate()?;

        Ok(image)
    }

    fn get_header(&self) -> Result<Header> {
        let len = size_of::<Header>();

        let header =
            Header::read_from_bytes(&self.kernel[0..len]).map_err(|_| Error::InvalidKernelImage)?;

        debug!(?header);

        Ok(header)
    }

    fn validate(&self) -> Result<()> {
        let len = size_of::<Header>();

        if self.kernel.len() < len {
            return Err(Error::InvalidKernelImage);
        }

        let header = self.get_header()?;

        if header.magic != 0x644d5241 {
            return Err(Error::InvalidKernelImage);
        }

        Ok(())
    }
}

pub struct AArch64BootParams {
    pub ram_base: u64,
    pub ram_size: u64,
}

impl<C> KernelLoader<C> for Image
where
    C: MemoryContainer,
{
    type BootParams = AArch64BootParams;

    fn load(
        &self,
        boot_params: &AArch64BootParams,
        memory: &mut MemoryAddressSpace<C>,
    ) -> Result<LoadResult> {
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
                .map_err(|_| Error::InvalidKernelImage)?
        };

        // Check 2MB alignment
        if !boot_params.ram_base.is_multiple_of(2 << 20) {
            return Err(Error::InvalidAddressAlignment);
        }

        let kernel_start = boot_params.ram_base + text_offset;
        let kernel_end = kernel_start + kernel_len as u64;

        let ram_base = boot_params.ram_base;
        let ram_end = boot_params.ram_base + boot_params.ram_size;
        if kernel_end > boot_params.ram_base + boot_params.ram_size {
            return Err(Error::OutOfMemory {
                kernel_end,
                memory_end: ram_end,
                memory_base: ram_base,
                memory_size: boot_params.ram_size,
            });
        }

        memory
            .copy_from_slice(kernel_start, &self.kernel, self.kernel.len())
            .map_err(|err| Error::CopyKernelFailed(err.to_string()))?;

        Ok(LoadResult {
            start_pc: kernel_start,
            kernel_start,
            kernel_len,
        })
    }
}
