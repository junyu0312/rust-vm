use std::fs;
use std::io;
use std::path::Path;

use anyhow::anyhow;
use anyhow::ensure;

struct Header {
    offset: usize,
    size: usize,
}

enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
}

impl Value {
    fn len(&self) -> usize {
        match self {
            Value::U8(_) => 1,
            Value::U16(_) => 2,
            Value::U32(_) => 4,
        }
    }

    fn as_u8(&self) -> anyhow::Result<u8> {
        match self {
            Value::U8(v) => Ok(*v),
            _ => Err(anyhow!("Value is not u8")),
        }
    }

    fn as_u16(&self) -> anyhow::Result<u16> {
        match self {
            Value::U16(v) => Ok(*v),
            _ => Err(anyhow!("Value is not u16")),
        }
    }

    fn as_u32(&self) -> anyhow::Result<u32> {
        match self {
            Value::U32(v) => Ok(*v),
            _ => Err(anyhow!("Value is not u32")),
        }
    }
}

const BOOT_FLAG: Header = Header {
    offset: 0x1fe,
    size: 2,
};
const LOADFLAGS: Header = Header {
    offset: 0x211,
    size: 1,
};
const HEAP_END_PTR: Header = Header {
    offset: 0x224,
    size: 2,
};
const CMD_LINE_PTR: Header = Header {
    offset: 0x228,
    size: 4,
};

pub struct BzImage {
    buf: Vec<u8>,
}

impl BzImage {
    pub fn read(path: &Path) -> io::Result<BzImage> {
        let buf = fs::read(path)?;
        Ok(BzImage { buf })
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn get_boot_flag(&self) -> anyhow::Result<u16> {
        self.get_header(&BOOT_FLAG)?.as_u16()
    }

    pub fn set_boot_flag(&mut self, flag: u16) -> anyhow::Result<()> {
        self.set_header(&BOOT_FLAG, Value::U16(flag))
    }

    pub fn get_loadflags(&self) -> anyhow::Result<u8> {
        self.get_header(&LOADFLAGS)?.as_u8()
    }

    pub fn set_loadflags(&mut self, flags: u8) -> anyhow::Result<()> {
        self.set_header(&LOADFLAGS, Value::U8(flags))
    }

    pub fn set_heap_end_ptr(&mut self, ptr: u16) -> anyhow::Result<()> {
        self.set_header(&HEAP_END_PTR, Value::U16(ptr))
    }

    pub fn set_cmd_line_ptr(&mut self, ptr: u32) -> anyhow::Result<()> {
        self.set_header(&CMD_LINE_PTR, Value::U32(ptr))
    }

    pub fn get_cmd_line_ptr(&self) -> anyhow::Result<u32> {
        self.get_header(&CMD_LINE_PTR)?.as_u32()
    }

    pub fn set_cmdline(&mut self, cmdline: &[u8], dst: u32) -> anyhow::Result<()> {
        let dst = dst as usize;
        self.buf.as_mut_slice()[dst..dst + cmdline.len()].copy_from_slice(cmdline);

        Ok(())
    }
}

impl BzImage {
    fn get_header(&self, header: &Header) -> anyhow::Result<Value> {
        let bytes = &self.buf[header.offset..header.offset + header.size];

        match header.size {
            1 => Ok(Value::U8(u8::from_le_bytes(bytes.try_into()?))),
            2 => Ok(Value::U16(u16::from_le_bytes(bytes.try_into()?))),
            4 => Ok(Value::U32(u32::from_le_bytes(bytes.try_into()?))),
            _ => Err(anyhow!("Unsupported header size")),
        }
    }

    fn set_header(&mut self, header: &Header, value: Value) -> anyhow::Result<()> {
        ensure!(header.size == value.len());
        ensure!(
            header.offset + header.size <= self.buf.len(),
            "Offset out of bounds"
        );

        match value {
            Value::U8(v) => self.buf.as_mut_slice()[header.offset..header.offset + header.size]
                .copy_from_slice(&v.to_le_bytes()),
            Value::U16(v) => self.buf.as_mut_slice()[header.offset..header.offset + header.size]
                .copy_from_slice(&v.to_le_bytes()),
            Value::U32(v) => self.buf.as_mut_slice()[header.offset..header.offset + header.size]
                .copy_from_slice(&v.to_le_bytes()),
        }

        Ok(())
    }
}
