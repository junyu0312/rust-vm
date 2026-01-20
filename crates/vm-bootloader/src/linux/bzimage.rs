use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::ensure;
use header::*;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::vcpu::arch::x86_64::X86Vcpu;
use vm_core::virt::Virt;

use crate::BootLoader;

mod header {
    use anyhow::anyhow;

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
        pub fn as_u8(&self) -> anyhow::Result<u8> {
            match self {
                Value::U8(v) => Ok(*v),
                _ => Err(anyhow!("Value is not u8")),
            }
        }

        pub fn as_u16(&self) -> anyhow::Result<u16> {
            match self {
                Value::U16(v) => Ok(*v),
                _ => Err(anyhow!("Value is not u16")),
            }
        }

        pub fn as_u32(&self) -> anyhow::Result<u32> {
            match self {
                Value::U32(v) => Ok(*v),
                _ => Err(anyhow!("Value is not u32")),
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
    fn read_header(&self, header: &Header) -> anyhow::Result<Value> {
        let bytes = &self.bzimage[header.offset..header.offset + header.size];

        match header.size {
            1 => Ok(Value::U8(u8::from_le_bytes(bytes.try_into()?))),
            2 => Ok(Value::U16(u16::from_le_bytes(bytes.try_into()?))),
            4 => Ok(Value::U32(u32::from_le_bytes(bytes.try_into()?))),
            _ => Err(anyhow!("Unsupported header size")),
        }
    }

    fn get_setup_sects(&self) -> anyhow::Result<u8> {
        self.read_header(&SETUP_SECTS)?.as_u8()
    }

    fn get_boot_flag(&self) -> anyhow::Result<u16> {
        self.read_header(&BOOT_FLAG)?.as_u16()
    }

    fn get_header(&self) -> anyhow::Result<u32> {
        self.read_header(&HEADER)?.as_u32()
    }

    fn get_version(&self) -> anyhow::Result<u16> {
        self.read_header(&VERSION)?.as_u16()
    }

    fn get_initrd_addr_max(&self) -> anyhow::Result<u32> {
        self.read_header(&INITRD_ADDR_MAX)?.as_u32()
    }

    fn get_cmdline_size(&self) -> anyhow::Result<u32> {
        self.read_header(&CMDLINE_SIZE)?.as_u32()
    }

    pub fn new(path: &Path, initrd: Option<&Path>, cmdline: Option<&str>) -> anyhow::Result<Self> {
        let bzimage = fs::read(path)?;
        let initrd = initrd.map(fs::read).transpose()?;
        let cmdline = cmdline.map(|s| s.to_string());

        Ok(BzImage {
            bzimage,
            initrd,
            cmdline,
        })
    }
}

impl<V> BootLoader<V> for BzImage
where
    V: Virt,
    V::Vcpu: X86Vcpu,
{
    fn install(
        &self,
        memory: &mut MemoryAddressSpace<V::Memory>,
        memory_size: usize,
        vcpu0: &mut V::Vcpu,
    ) -> anyhow::Result<()> {
        ensure!(self.get_boot_flag()? == 0xAA55, "Invalid boot_flag");

        ensure!(
            self.get_header()? == u32::from_le_bytes("HdrS".as_bytes().try_into()?),
            "Invalid header"
        );

        let version = self.get_version()?;
        ensure!(version >= MINIMAL_VERSION, "Invalid version");

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
                memory.copy_from_slice(setup_start_gpa, &self.bzimage[..], setup_sects_size)?;
            }

            {
                // copy kernel
                let buf = &self.bzimage[setup_sects_size..];
                memory.copy_from_slice(KERNEL_START as u64, buf, buf.len())?;
            }

            {
                // copy cmdline
                if let Some(cmdline) = &self.cmdline {
                    let cmdline_size = self.get_cmdline_size()? as usize;
                    let cstr = CString::new(cmdline.to_string())?;

                    let len = cstr.count_bytes();
                    ensure!(len < cmdline_size, "Cmdline too long");

                    memory.memset(CMDLINE_OFFSET as u64, 0, cmdline_size)?;
                    memory.copy_from_slice(
                        CMDLINE_OFFSET as u64,
                        cstr.as_bytes_with_nul(),
                        cstr.count_bytes(),
                    )?;
                }
            }

            {
                // copy initramfs
                if let Some(initrd) = &self.initrd {
                    let initrd_address = memory_size.min(self.get_initrd_addr_max()? as usize);
                    let initrd_address = initrd_address as u32 - initrd.len() as u32;

                    memory.copy_from_slice(initrd_address as u64, initrd, initrd.len())?;

                    unsafe {
                        let ptr = memory
                            .gpa_to_hva(setup_start_gpa + RAMDISK_IMAGE.offset as u64)?
                            as *mut u32;
                        *ptr = initrd_address;
                    }

                    unsafe {
                        let ptr = memory.gpa_to_hva(setup_start_gpa + RAMDISK_SIZE.offset as u64)?
                            as *mut u32;
                        *ptr = initrd.len() as u32;
                    }
                }
            }

            unsafe {
                let ptr =
                    memory.gpa_to_hva(setup_start_gpa + CMD_LINE_PTR.offset as u64)? as *mut u32;
                *ptr = CMDLINE_OFFSET;
            }
            unsafe {
                let ptr =
                    memory.gpa_to_hva(setup_start_gpa + HEAP_END_PTR.offset as u64)? as *mut u16;
                *ptr = 0xfe00;
            }
            unsafe {
                let ptr = memory.gpa_to_hva(setup_start_gpa + TYPE_OF_LOADER.offset as u64)?;
                *ptr = 0xff; // undefined
            }
            unsafe {
                let ptr = memory.gpa_to_hva(setup_start_gpa + LOADFLAGS.offset as u64)?;
                *ptr |= 0x80;
            }

            {
                // To meet kvmtool bios
                {
                    const VGA_ROM_BEGIN: u64 = 0x000c0000;
                    const VGA_ROM_OEM_STRING: u64 = VGA_ROM_BEGIN;
                    const VGA_ROM_OEM_STRING_SIZE: usize = 16;
                    const VGA_ROM_MODES: u64 = VGA_ROM_OEM_STRING + VGA_ROM_OEM_STRING_SIZE as u64;

                    memory.copy_from_slice(
                        VGA_ROM_BEGIN,
                        &[0; VGA_ROM_OEM_STRING_SIZE],
                        VGA_ROM_OEM_STRING_SIZE,
                    )?;
                    let s = CString::from_str("KVM VESA")?;
                    memory.copy_from_slice(VGA_ROM_BEGIN, s.as_bytes(), s.count_bytes())?;

                    memory.copy_from_slice(VGA_ROM_MODES, &0x0112u16.to_le_bytes(), 2)?;
                    memory.copy_from_slice(VGA_ROM_MODES + 2, &0x0ffffu16.to_le_bytes(), 2)?;
                }
            }
        }

        {
            let mut regs = vcpu0.get_regs()?;
            regs.rip = IP as u64 + 0x200;
            regs.rsp = SP as u64;
            regs.rbp = SP as u64;
            regs.rflags = 0x2;
            vcpu0.set_regs(&regs)?;

            let mut sregs = vcpu0.get_sregs()?;
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
            vcpu0.set_sregs(&sregs)?;
        }

        Ok(())
    }
}
