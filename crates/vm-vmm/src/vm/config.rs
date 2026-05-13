use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use vm_device::device::Device;

#[derive(Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub memory_size: usize,
    pub vcpus: usize,
    pub devices: Vec<Device>,
    pub gdb_port: Option<u16>,
    pub kernel: PathBuf,
    pub initramfs: Option<PathBuf>,
    pub cmdline: Option<String>,
}
