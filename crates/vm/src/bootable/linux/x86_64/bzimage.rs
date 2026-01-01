use std::ffi::CString;
use std::fs;
use std::path::Path;

use anyhow::anyhow;
use anyhow::ensure;
use header::*;
use kvm_bindings::KVM_GUESTDBG_ENABLE;
use kvm_bindings::KVM_GUESTDBG_SINGLESTEP;
use kvm_bindings::KVM_MAX_CPUID_ENTRIES;
use kvm_bindings::kvm_guest_debug;
use kvm_bindings::kvm_guest_debug_arch;

use crate::bootable::Bootable;
use crate::kvm::vm::KvmVm;

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
    pub const HEAP_END_PTR: Header = Header {
        offset: 0x224,
        size: 2,
    };
    pub const CMD_LINE_PTR: Header = Header {
        offset: 0x228,
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
const KERNEL_START: u32 = 0x100000;

fn to_gpa(cs: u16, ip: u16) -> u32 {
    ((cs as u32) << 4) + ip as u32
}

pub struct BzImage {
    bzimage: Vec<u8>,
    #[allow(dead_code)]
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

impl Bootable for BzImage {
    fn init(&mut self, vm: &mut KvmVm) -> anyhow::Result<()> {
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
            let memory = vm
                .memory_regions
                .get_mut()
                .ok_or_else(|| anyhow!("Memory is not initialized"))?;

            let setup_start_gpa = to_gpa(CS, IP) as usize;

            // boot sector + setup code
            let setup_sects_size = (setup_sects + 1) as usize * 512;

            {
                // copy setup
                memory.copy_from_slice(setup_start_gpa, &self.bzimage[..], setup_sects_size)?;
            }

            {
                // copy kernel
                let buf = &self.bzimage[setup_sects_size..];
                memory.copy_from_slice(KERNEL_START as usize, buf, buf.len())?;
            }

            {
                // copy cmdline
                if let Some(cmdline) = &self.cmdline {
                    let cmdline_size = self.get_cmdline_size()? as usize;
                    let cstr = CString::new(cmdline.to_string())?;

                    let len = cstr.count_bytes();
                    ensure!(len < cmdline_size, "Cmdline too long");

                    memory.memset(CMDLINE_OFFSET as usize, 0, cmdline_size)?;
                    memory.copy_from_slice(
                        CMDLINE_OFFSET as usize,
                        cstr.as_bytes_with_nul(),
                        cstr.count_bytes(),
                    )?;
                }
            }

            unsafe {
                let ptr = memory.gpa_to_ptr(setup_start_gpa + CMD_LINE_PTR.offset)? as *mut u32;
                *ptr = CMDLINE_OFFSET;
            }
            unsafe {
                let ptr = memory.gpa_to_ptr(setup_start_gpa + HEAP_END_PTR.offset)? as *mut u16;
                *ptr = 0xfe00;
            }
            unsafe {
                let ptr = memory.gpa_to_ptr(setup_start_gpa + TYPE_OF_LOADER.offset)?;
                *ptr = 0xff; // undefined
            }
            unsafe {
                let ptr = memory.gpa_to_ptr(setup_start_gpa + LOADFLAGS.offset)?;
                *ptr |= 0x80;
            }
        }

        {
            let vcpus = vm
                .vcpus
                .get_mut()
                .ok_or_else(|| anyhow!("Cpu is not initialized"))?;
            let vcpu0 = vcpus.get_mut(0).ok_or_else(|| anyhow!("No cpu0"))?;

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

            vcpu0.set_cpuid2(&vm.kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)?)?;

            vcpu0.set_guest_debug(&kvm_guest_debug {
                control: KVM_GUESTDBG_ENABLE | KVM_GUESTDBG_SINGLESTEP,
                pad: 0,
                arch: kvm_guest_debug_arch { debugreg: [0; 8] },
            })?;
        }

        Ok(())
    }
}
