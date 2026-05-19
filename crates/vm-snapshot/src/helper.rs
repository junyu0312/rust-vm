use std::io::Read;
use std::io::Write;

pub fn write_bool(writer: &mut dyn Write, v: bool) -> std::io::Result<()> {
    if v {
        writer.write_all(&[1])
    } else {
        writer.write_all(&[0])
    }
}

pub fn write_u8(writer: &mut dyn Write, v: u8) -> std::io::Result<()> {
    writer.write_all(&v.to_le_bytes())
}

pub fn write_u16(writer: &mut dyn Write, v: u16) -> std::io::Result<()> {
    writer.write_all(&v.to_le_bytes())
}

pub fn write_u32(writer: &mut dyn Write, v: u32) -> std::io::Result<()> {
    writer.write_all(&v.to_le_bytes())
}

pub fn write_u64(writer: &mut dyn Write, v: u64) -> std::io::Result<()> {
    writer.write_all(&v.to_le_bytes())
}

pub fn write_usize(writer: &mut dyn Write, v: usize) -> std::io::Result<()> {
    writer.write_all(&v.to_le_bytes())
}

pub fn write_option_u16(writer: &mut dyn Write, value: &Option<u16>) -> std::io::Result<()> {
    match value {
        Some(v) => {
            writer.write_all(&[1])?;
            writer.write_all(&v.to_le_bytes())?;
        }
        None => {
            writer.write_all(&[0])?;
        }
    }

    Ok(())
}

pub fn write_option_u32(writer: &mut dyn Write, value: &Option<u32>) -> std::io::Result<()> {
    match value {
        Some(v) => {
            writer.write_all(&[1])?;
            writer.write_all(&v.to_le_bytes())?;
        }
        None => {
            writer.write_all(&[0])?;
        }
    }

    Ok(())
}

pub fn read_u8(reader: &mut dyn Read) -> std::io::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_u16(reader: &mut dyn Read) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub fn read_u32(reader: &mut dyn Read) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_u64(reader: &mut dyn Read) -> std::io::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn read_usize(reader: &mut dyn Read) -> std::io::Result<usize> {
    let mut buf = [0u8; size_of::<usize>()];
    reader.read_exact(&mut buf)?;
    Ok(usize::from_le_bytes(buf))
}

pub fn read_option_u16(reader: &mut dyn Read) -> std::io::Result<Option<u16>> {
    if read_u8(reader)? == 1 {
        Ok(Some(read_u16(reader)?))
    } else {
        Ok(None)
    }
}

pub fn read_option_u32(reader: &mut dyn Read) -> std::io::Result<Option<u32>> {
    if read_u8(reader)? == 1 {
        Ok(Some(read_u32(reader)?))
    } else {
        Ok(None)
    }
}
