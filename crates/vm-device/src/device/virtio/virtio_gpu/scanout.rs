use vm_virtio::types::device::gpu::request::virtio_gpu_scanout::VirtioGpuRect;

pub struct Scanout {
    pub width: u32,
    pub height: u32,
    /// The driver can use resource_id = 0 to disable a scanout.
    pub resource: u32,
    pub rect: Option<VirtioGpuRect>,
}
