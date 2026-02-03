use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid memory format({0})")]
    InvalidMemoryFmt(String),
    #[error("memory too large")]
    MemoryTooLarge(String),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Accel {
    #[cfg(feature = "kvm")]
    Kvm,
    #[cfg(feature = "hvp")]
    Hvp,
}

#[derive(Debug, Parser)]
pub struct Command {
    #[arg(short, long)]
    pub cpus: usize,

    #[arg(short, long)]
    pub memory: String,

    #[arg(short, long)]
    pub accel: Accel,

    #[arg(short, long)]
    pub kernel: PathBuf,

    #[arg(short, long)]
    pub cmdline: Option<String>,

    #[arg(short, long)]
    pub initramfs: Option<PathBuf>,
}

pub fn parse_memory(s: &str) -> Result<usize, Error> {
    let s = s.trim().to_lowercase();

    let pos = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());

    let num_part = &s[..pos];
    let unit_part = &s[pos..];

    let num = num_part
        .parse::<usize>()
        .map_err(|_| Error::InvalidMemoryFmt(s.to_string()))?;

    let shift = match unit_part.trim() {
        "" => 0,
        "k" => 10,
        "m" => 20,
        "g" => 30,
        _ => return Err(Error::InvalidMemoryFmt(s.to_string())),
    };

    let bytes = num
        .checked_shl(shift)
        .ok_or(Error::MemoryTooLarge(s.to_string()))?;

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory() -> anyhow::Result<()> {
        assert_eq!(parse_memory("1")?, 1);
        assert_eq!(parse_memory("1K")?, 1 << 10);
        assert_eq!(parse_memory("1M")?, 1 << 20);
        assert_eq!(parse_memory("1G")?, 1 << 30);

        Ok(())
    }
}
