use std::fs;
use std::path::Path;

use header::*;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;

use crate::kernel_loader::Error;
use crate::kernel_loader::KernelLoader;
use crate::kernel_loader::LoadResult;
use crate::kernel_loader::Result;

mod header {
    pub struct Header {
        pub offset: usize,
        pub size: usize,
    }

    pub const SETUP_SECTS: Header = Header {
        offset: 0x1f1,
        size: 1,
    };
    pub const BOOT_FLAG: Header = Header {
        offset: 0x1fe,
        size: 2,
    };
    pub const HEADER: Header = Header {
        offset: 0x202,
        size: 4,
    };
    pub const VERSION: Header = Header {
        offset: 0x206,
        size: 2,
    };
    pub const TYPE_OF_LOADER: Header = Header {
        offset: 0x210,
        size: 1,
    };
    pub const LOADFLAGS: Header = Header {
        offset: 0x211,
        size: 1,
    };
    pub const RAMDISK_IMAGE: Header = Header {
        offset: 0x218,
        size: 4,
    };
    pub const RAMDISK_SIZE: Header = Header {
        offset: 0x21c,
        size: 4,
    };
    pub const HEAP_END_PTR: Header = Header {
        offset: 0x224,
        size: 2,
    };
    pub const CMD_LINE_PTR: Header = Header {
        offset: 0x228,
        size: 4,
    };
    pub const INITRD_ADDR_MAX: Header = Header {
        offset: 0x22c,
        size: 4,
    };
    pub const CMDLINE_SIZE: Header = Header {
        offset: 0x238,
        size: 4,
    };

    pub const MINIMAL_VERSION: u16 = 0x206;

    pub enum Value {
        U8(u8),
        U16(u16),
        U32(u32),
    }

    impl Value {
        pub fn as_u8(&self) -> u8 {
            match self {
                Value::U8(v) => *v,
                _ => unreachable!(),
            }
        }

        pub fn as_u16(&self) -> u16 {
            match self {
                Value::U16(v) => *v,
                _ => unreachable!(),
            }
        }

        pub fn as_u32(&self) -> u32 {
            match self {
                Value::U32(v) => *v,
                _ => unreachable!(),
            }
        }
    }
}

const CS: u16 = 0x1000;
const IP: u16 = 0x0000;
const SP: u16 = 0x8000;
const CMDLINE_OFFSET: u32 = 0x20000;
pub const KERNEL_START: u32 = 0x100000;

fn to_gpa(cs: u16, ip: u16) -> u32 {
    ((cs as u32) << 4) + ip as u32
}

pub struct BzImage {
    bzimage: Vec<u8>,
    initrd: Option<Vec<u8>>,
    cmdline: Option<String>,
}

impl BzImage {
    fn read_header(&self, header: &Header) -> Result<Value> {
        let bytes = &self.bzimage[header.offset..header.offset + header.size];

        match header.size {
            1 => Ok(Value::U8(u8::from_le_bytes(
                bytes.try_into().map_err(|_| Error::InvalidKernelImage)?,
            ))),
            2 => Ok(Value::U16(u16::from_le_bytes(
                bytes.try_into().map_err(|_| Error::InvalidKernelImage)?,
            ))),
            4 => Ok(Value::U32(u32::from_le_bytes(
                bytes.try_into().map_err(|_| Error::InvalidKernelImage)?,
            ))),
            _ => Err(Error::InvalidKernelImage),
        }
    }

    fn get_setup_sects(&self) -> Result<u8> {
        Ok(self.read_header(&SETUP_SECTS)?.as_u8())
    }

    fn get_boot_flag(&self) -> Result<u16> {
        Ok(self.read_header(&BOOT_FLAG)?.as_u16())
    }

    fn get_header(&self) -> Result<u32> {
        Ok(self.read_header(&HEADER)?.as_u32())
    }

    fn get_version(&self) -> Result<u16> {
        Ok(self.read_header(&VERSION)?.as_u16())
    }

    fn get_initrd_addr_max(&self) -> Result<u32> {
        Ok(self.read_header(&INITRD_ADDR_MAX)?.as_u32())
    }

    fn get_cmdline_size(&self) -> Result<u32> {
        Ok(self.read_header(&CMDLINE_SIZE)?.as_u32())
    }

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
}

impl<C> KernelLoader<C> for BzImage
where
    C: MemoryContainer,
{
    type BootParams = ();

    fn load(
        &self,
        _boot_params: &Self::BootParams,
        _memory: &mut MemoryAddressSpace<C>,
    ) -> Result<LoadResult> {
        todo!()
    }

    /*
    fn install(
        &self,
        _ram_base: u64,
        memory: &mut MemoryAddressSpace<V::Memory>,
        memory_size: usize,
        vcpu0: &mut V::Vcpu,
    ) -> Result<(), Error> {
        if self.get_boot_flag()? != 0xAA55 {
            return Err(Error::InvalidKernelImage);
        }

        if self.get_header()? != u32::from_le_bytes("HdrS".as_bytes().try_into().unwrap()) {
            return Err(Error::InvalidKernelImage);
        }

        let version = self.get_version()?;
        if version < MINIMAL_VERSION {
            return Err(Error::InvalidKernelImage);
        }

        let mut setup_sects = self.get_setup_sects()?;
        if setup_sects == 0 {
            setup_sects = 4;
        }

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
