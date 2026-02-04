use vm_virtio::v2::device::DeviceId;
use vm_virtio::v2::device::VirtIoDevice;
use vm_virtio::v2::transport::mmio::VirtIoMmioTransport;
use vm_virtio::v2::types::device_features::VIRTIO_F_VERSION_1;

pub struct VirtIoBlkDevice {
    irq: u32,
}

impl VirtIoBlkDevice {
    pub fn new(irq: u32) -> Self {
        VirtIoBlkDevice { irq }
    }
}

impl VirtIoDevice for VirtIoBlkDevice {
    const NAME: &str = "virtio-blk";
    const DEVICE_ID: u32 = DeviceId::Blk as u32;
    const VIRT_QUEUES_SIZE_MAX: &[u32] = &[512];
    const DEVICE_FEATURES: u64 = 1 << VIRTIO_F_VERSION_1;

    fn irq(&self) -> Option<u32> {
        Some(self.irq)
    }

    fn reset(&mut self) {}
}

pub type VirtIoMmioBlkDevice = VirtIoMmioTransport<VirtIoBlkDevice>;
