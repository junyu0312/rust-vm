use std::path::PathBuf;

use serde::Deserialize;
use vm_vmm::vm::config::VmConfig;

use crate::cmd::device::Device;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct CreateArgs {
    cpus: usize,

    memory: String,

    #[serde(default)]
    device: Vec<Device>,

    kernel: PathBuf,

    cmdline: Option<String>,

    initramfs: Option<PathBuf>,

    gdb: Option<u16>,
}

impl TryInto<VmConfig> for CreateArgs {
    type Error = Error;

    fn try_into(self) -> Result<VmConfig, Self::Error> {
        let vm_config = VmConfig {
            memory_size: parse_memory(&self.memory)?,
            vcpus: self.cpus,
            devices: self.device.into_iter().map(Into::into).collect(),
            gdb_port: self.gdb,
            kernel: self.kernel,
            initramfs: self.initramfs,
            cmdline: self.cmdline,
        };

        Ok(vm_config)
    }
}

fn parse_memory(s: &str) -> Result<usize, Error> {
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
