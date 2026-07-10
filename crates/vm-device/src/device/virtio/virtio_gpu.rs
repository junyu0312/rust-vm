use std::sync::Arc;

use tokio::sync::Mutex;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::result::VirtioError;
use vm_virtio::transport::pci::VirtioPciDevice;
use vm_virtio::types::device::gpu::VirtioGpuConfig;
use vm_virtio::types::device::gpu::VirtioGpuConfigOffset;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use zerocopy::IntoBytes;

use crate::device::virtio::virtio_gpu::controlq_handler::ControlqHandler;
use crate::device::virtio::virtio_gpu::cursorq_handler::CursorqHandler;
use crate::device::virtio::virtio_gpu::scanout::Scanout;

mod controlq_handler;
mod cursorq_handler;
mod scanout;

// Queue for sending control commands
const CONTROLQ: u16 = 1024;
// Queue for sending cursor updates
const CURSORQ: u16 = 1024;

pub struct VirtioGpu {
    memory: Arc<MemoryAddressSpace>,
    cfg: VirtioGpuConfig,
    scanout: Arc<Mutex<Vec<Scanout>>>,
}

impl VirtioGpu {
    pub fn new(memory: Arc<MemoryAddressSpace>) -> Self {
        VirtioGpu {
            memory,
            cfg: VirtioGpuConfig {
                num_scanouts: 1,
                num_capsets: 0,
                blob_alignment: 1,
                ..Default::default()
            },
            scanout: Arc::new(Mutex::new(vec![Scanout {
                width: 1024,
                height: 768,
            }])),
        }
    }
}

impl VirtioDevice for VirtioGpu {
    const NAME: &str = "virtio-gpu";
    const DEVICE_ID: u16 = DeviceId::Gpu as u16;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<u16> {
        vec![CONTROLQ, CURSORQ]
    }

    fn reset(&mut self) {}

    fn virtqueue_handler(&self, queue_sel: u16) -> Option<Box<dyn VirtqueueHandler>> {
        if queue_sel == 0 {
            return Some(Box::new(ControlqHandler {
                scanouts: self.scanout.clone(),
                memory: self.memory.clone(),
            }));
        }

        if queue_sel == 1 {
            return Some(Box::new(CursorqHandler));
        }

        return None;
    }

    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<(), VirtioError> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<(), VirtioError> {
        match VirtioGpuConfigOffset::from_repr(offset) {
            Some(VirtioGpuConfigOffset::EventsClear) => {
                let val = u32::from_le_bytes(buf.try_into().unwrap());
                self.cfg.events_read &= !val;
            }
            _ => (),
        }

        self.cfg.as_mut_bytes()[offset..offset + buf.len()].copy_from_slice(buf);

        Ok(())
    }
}

impl VirtioPciDevice for VirtioGpu {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioGpuConfig>();
    const CLASS_CODE: u32 = 0x030000;
    const IRQ_PIN: u8 = InterruptPin::INTA as u8;
}
