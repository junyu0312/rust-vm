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
use crate::kernel_loader::linux::image::layout::DEFAULT_TEXT_OFFSET;

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
    kernel: Vec<u8>,
    initrd: Option<Vec<u8>>,
    cmdline: Option<String>,
}

impl Image {
    pub fn new(kernel: &Path, initrd: Option<&Path>, cmdline: Option<&str>) -> Result<Self, Error> {
        let kernel = fs::read(kernel).map_err(|_| Error::ReadFailed)?;
        let initrd = initrd
            .map(fs::read)
            .transpose()
            .map_err(|_| Error::ReadFailed)?;
        let cmdline = cmdline.map(|s| s.to_string());

        let image = Image {
            kernel,
            initrd,
            cmdline,
        };

        image.validate()?;

        Ok(image)
    }

    fn get_header(&self) -> Result<Header, Error> {
        let len = size_of::<Header>();

        let header =
            Header::read_from_bytes(&self.kernel[0..len]).map_err(|_| Error::InvalidKernelImage)?;

        debug!(?header);

        Ok(header)
    }

    fn validate(&self) -> Result<(), Error> {
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
    ) -> Result<LoadResult, Error> {
        let header = self.get_header()?;

        let text_offset = if header.image_size == 0 {
            DEFAULT_TEXT_OFFSET
        } else {
            header.text_offset
        };

        let image_size = if header.image_size == 0 {
            self.kernel.len() as u64
        } else {
            header.image_size
        };

        // Check 2MB alignment
        if !boot_params.ram_base.is_multiple_of(2 << 20) {
            return Err(Error::InvalidAddressAlignment);
        }

        let kernel_start = boot_params.ram_base + text_offset;
        let kernel_end = kernel_start + image_size;

        memory
            .copy_from_slice(kernel_start, &self.kernel, self.kernel.len())
            .map_err(|err| Error::CopyKernelFailed(err.to_string()))?;

        if let Some(_initrd) = &self.initrd {
            todo!()
        }

        if let Some(_cmdline) = &self.cmdline {
            todo!()
        }

        Ok(LoadResult {
            start_pc: kernel_start,
            kernel_start,
            kernel_end,
        })
    }
}
