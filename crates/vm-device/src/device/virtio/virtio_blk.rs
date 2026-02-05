use vm_virtio::device::VirtIoDevice;
use vm_virtio::result::Result;
use vm_virtio::transport::mmio::VirtIoMmioTransport;
use vm_virtio::types::device_config::blk::config::VirtioBlkConfig;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use zerocopy::IntoBytes;

pub struct VirtIoBlkDevice {
    irq: u32,
    cfg: VirtioBlkConfig,
}

impl VirtIoBlkDevice {
    pub fn new(irq: u32) -> Self {
        let cfg = VirtioBlkConfig::default();
        VirtIoBlkDevice { irq, cfg }
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

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + len]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()> {
        self.cfg.as_mut_bytes()[offset..len].copy_from_slice(buf);
        Ok(())
    }
}

pub type VirtIoMmioBlkDevice = VirtIoMmioTransport<VirtIoBlkDevice>;
