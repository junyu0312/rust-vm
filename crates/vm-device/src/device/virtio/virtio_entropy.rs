use std::io::Read;
use std::io::Write;
use std::slice;
use std::sync::Arc;

use async_trait::async_trait;
use rand::Rng;
use vm_core::device::error::DeviceSnapshotError;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::result::VirtioError;
use vm_virtio::transport::pci::VirtioPciDevice;
use vm_virtio::types::device::entropy::VirtioEntropyConfig;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use vm_virtio::virtqueue::virtq_desc_table::VIRTQ_DESC_F_WRITE;
use vm_virtio::virtqueue::virtq_desc_table::VirtqDescTableRef;

struct RequestqHandler {
    memory: Arc<MemoryAddressSpace>,
}

#[async_trait]
impl VirtqueueHandler for RequestqHandler {
    async fn handle_desc(&self, desc_ring: &VirtqDescTableRef, desc_id: u16) -> u32 {
        let desc = desc_ring.get(desc_id);
        let len = desc.len;

        let mut rng = rand::rng();

        let buf = desc.addr(&self.memory).unwrap().as_ptr();

        assert!(desc.flags & VIRTQ_DESC_F_WRITE != 0);

        unsafe {
            rng.fill_bytes(slice::from_raw_parts_mut(buf, len as usize));
        }

        len
    }
}

pub struct VirtioEntropy {
    memory: Arc<MemoryAddressSpace>,
}

impl VirtioEntropy {
    pub fn new(memory: Arc<MemoryAddressSpace>) -> Self {
        VirtioEntropy { memory }
    }
}

impl VirtioDevice for VirtioEntropy {
    const NAME: &str = "virtio-entropy";
    const DEVICE_ID: u16 = DeviceId::Entropy as u16;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<u16> {
        vec![8]
    }

    fn reset(&mut self) {}

    fn virtqueue_handler(&self, queue: u16) -> Option<Box<dyn VirtqueueHandler>> {
        if queue != 0 {
            return None;
        }

        Some(Box::new(RequestqHandler {
            memory: self.memory.clone(),
        }))
    }

    fn read_config(&self, _offset: usize, _buf: &mut [u8]) -> Result<(), VirtioError> {
        Ok(()) // no cfg for entropy device
    }

    fn write_config(&mut self, _offset: usize, _buf: &[u8]) -> Result<(), VirtioError> {
        Ok(()) // no cfg for entropy device
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }
}

impl VirtioPciDevice for VirtioEntropy {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioEntropyConfig>();
    const CLASS_CODE: u32 = 0xff0000;
    const IRQ_PIN: u8 = InterruptPin::INTB as u8;
}
