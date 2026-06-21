use std::collections::HashSet;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use vm_core::device::error::DeviceSnapshotError;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_snapshot::helper::read_u32;
use vm_snapshot::helper::read_usize;
use vm_snapshot::helper::write_u32;
use vm_snapshot::helper::write_usize;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::result::VirtioError;
use vm_virtio::transport::pci::VirtioPciDevice;
use vm_virtio::types::device::balloon_tranditional::VirtioBalloonTranditionalConfig;
use vm_virtio::types::device::balloon_tranditional::VirtioBalloonTranditionalVirtqueue;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use vm_virtio::virtqueue::virtq_desc_table::VirtqDescTableRef;
use zerocopy::IntoBytes;

const INFLATEQ_QUEUE_SIZE_MAX: u16 = 512;
const DEFLATEQ_QUEUE_SIZE_MAX: u16 = 512;

struct InflateqHandler {
    balloon: Arc<Mutex<HashSet<u32>>>,
    memory: Arc<MemoryAddressSpace>,
}

#[async_trait]
impl VirtqueueHandler for InflateqHandler {
    async fn handle_desc(&self, desc_ring: &VirtqDescTableRef, desc_id: u16) -> u32 {
        let desc = desc_ring.get(desc_id);
        let len = desc.len;
        assert!(len.is_multiple_of(4));

        let array: &[u32] = unsafe {
            std::slice::from_raw_parts(
                desc.addr(&self.memory).unwrap().as_ptr() as *const u32,
                (len / 4) as usize,
            )
        };

        for &pfn in array {
            assert!(self.balloon.lock().await.insert(pfn));
            let gpa = (pfn as u64) << 12;
            let _hva = self.memory.gpa_to_hva(gpa).unwrap();
            // TODO: mmap
        }

        len
    }
}

struct DeflateqHandler {
    balloon: Arc<Mutex<HashSet<u32>>>,
    memory: Arc<MemoryAddressSpace>,
}

#[async_trait]
impl VirtqueueHandler for DeflateqHandler {
    async fn handle_desc(&self, desc_ring: &VirtqDescTableRef, desc_id: u16) -> u32 {
        let desc = desc_ring.get(desc_id);
        let len = desc.len;
        assert!(len.is_multiple_of(4));

        let array: &[u32] = unsafe {
            std::slice::from_raw_parts(
                desc.addr(&self.memory).unwrap().as_ptr() as *const u32,
                (len / 4) as usize,
            )
        };

        for pfn in array {
            assert!(self.balloon.lock().await.remove(pfn));
            let gpa = (*pfn as u64) << 12;
            let _hva = self.memory.gpa_to_hva(gpa).unwrap();
            // TODO: mmap
        }

        len
    }
}

pub struct VirtioBalloonTranditional {
    cfg: Arc<Mutex<VirtioBalloonTranditionalConfig>>,
    balloon: Arc<Mutex<HashSet<u32>>>,
    memory: Arc<MemoryAddressSpace>,
}

impl VirtioBalloonTranditional {
    pub fn new(memory: Arc<MemoryAddressSpace>) -> Self {
        VirtioBalloonTranditional {
            cfg: Default::default(),
            balloon: Default::default(),
            memory,
        }
    }

    pub fn get_cfg(&self) -> Arc<Mutex<VirtioBalloonTranditionalConfig>> {
        self.cfg.clone()
    }
}

impl VirtioDevice for VirtioBalloonTranditional {
    const NAME: &str = "virtio-balloon-tranditional";
    const DEVICE_ID: u16 = DeviceId::Balloon as u16;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<u16> {
        vec![INFLATEQ_QUEUE_SIZE_MAX, DEFLATEQ_QUEUE_SIZE_MAX]
    }

    fn reset(&mut self) {}

    fn virtqueue_handler(&self, queue: u16) -> Option<Box<dyn VirtqueueHandler>> {
        match VirtioBalloonTranditionalVirtqueue::from_repr(queue) {
            Some(virtq) => match virtq {
                VirtioBalloonTranditionalVirtqueue::Inflateq => Some(Box::new(InflateqHandler {
                    balloon: self.balloon.clone(),
                    memory: self.memory.clone(),
                })),
                VirtioBalloonTranditionalVirtqueue::Defalteq => Some(Box::new(DeflateqHandler {
                    balloon: self.balloon.clone(),
                    memory: self.memory.clone(),
                })),
                VirtioBalloonTranditionalVirtqueue::Statsq => None,
                VirtioBalloonTranditionalVirtqueue::FreePageVq => None,
                VirtioBalloonTranditionalVirtqueue::ReportingVq => None,
            },
            None => None,
        }
    }

    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<(), VirtioError> {
        buf.copy_from_slice(&self.cfg.blocking_lock().as_bytes()[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<(), VirtioError> {
        self.cfg.blocking_lock().as_mut_bytes()[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        writer.write_all(self.cfg.blocking_lock().as_bytes())?;

        {
            let balloon = self.balloon.blocking_lock();
            write_usize(writer, balloon.len())?;
            for v in balloon.iter() {
                write_u32(writer, *v)?;
            }
        }

        Ok(())
    }

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        reader.read_exact(self.cfg.blocking_lock().as_mut_bytes())?;

        {
            let mut balloon = self.balloon.blocking_lock();
            let len = read_usize(reader)?;
            for _ in 0..len {
                balloon.insert(read_u32(reader)?);
            }
        }

        Ok(())
    }
}

impl VirtioPciDevice for VirtioBalloonTranditional {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize =
        size_of::<VirtioBalloonTranditionalConfig>();
    const CLASS_CODE: u32 = 0xff0000;
    const IRQ_PIN: u8 = InterruptPin::INTA as u8;
}
