use vm_device::device::Device;

pub struct VmConfig {
    pub memory_size: usize,
    pub vcpus: usize,
    pub devices: Vec<Device>,
    pub gdb_port: Option<u16>,
}
