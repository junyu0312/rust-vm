use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use vm_core::device::error::DeviceSnapshotError;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::result::VirtioError;
use vm_virtio::transport::pci::VirtioPciDevice;
use vm_virtio::types::device::blk::config::VirtioBlkConfig;
use vm_virtio::types::device::blk::req::VirtioBlkReq;
use vm_virtio::types::device::blk::req::VirtioBlkReqType;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use vm_virtio::virtqueue::virtq_desc_table::VirtqDescTableRef;
use zerocopy::IntoBytes;

struct Requestq0Handler {
    memory: Arc<MemoryAddressSpace>,
}

#[async_trait]
impl VirtqueueHandler for Requestq0Handler {
    async fn handle_desc(&self, desc_ring: &VirtqDescTableRef, desc_id: u16) -> u32 {
        let desc_entry = desc_ring.get(desc_id);
        let req = desc_entry.addr(&self.memory).unwrap();
        let req = unsafe { &*(req.as_ptr() as *const VirtioBlkReq) };

        match req.r#type {
            VirtioBlkReqType::VirtioBlkTIn => {
                let chains = desc_ring.get_chain(desc_id);

                let data = chains[1];
                let data_hva = data.addr(&self.memory).unwrap();
                let data_len = data.len;
                unsafe { data_hva.write_bytes(0xff, data_len.try_into().unwrap()) };

                let status = chains[2];
                let mut status_hva = status.addr(&self.memory).unwrap();
                *unsafe { status_hva.as_mut() } = 0;

                data_len + 1
            }
            VirtioBlkReqType::VirtioBlkTOut => todo!(),
            VirtioBlkReqType::VirtioBlkTFlush => todo!(),
            VirtioBlkReqType::VirtioBlkTGetId => todo!(),
            VirtioBlkReqType::VirtioBlkTGetLifetime => todo!(),
            VirtioBlkReqType::VirtioBlkTDiscard => todo!(),
            VirtioBlkReqType::VirtioBlkTWriteZeroes => todo!(),
            VirtioBlkReqType::VirtioBlkTSecureErase => todo!(),
        }
    }
}

pub struct VirtioBlkDevice {
    cfg: VirtioBlkConfig,
    memory: Arc<MemoryAddressSpace>,
}

impl VirtioBlkDevice {
    pub fn new(memory: Arc<MemoryAddressSpace>) -> Self {
        let cfg = VirtioBlkConfig {
            capacity: 50,
            ..Default::default()
        };

        VirtioBlkDevice { cfg, memory }
    }
}

impl VirtioDevice for VirtioBlkDevice {
    const NAME: &str = "virtio-blk";
    const DEVICE_ID: u16 = DeviceId::Blk as u16;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<u16> {
        vec![512]
    }

    fn reset(&mut self) {}

    fn virtqueue_handler(&self, queue_sel: u16) -> Option<Box<dyn VirtqueueHandler>> {
        if queue_sel != 0 {
            return None;
        }

        Some(Box::new(Requestq0Handler {
            memory: self.memory.clone(),
        }))
    }

    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<(), VirtioError> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<(), VirtioError> {
        self.cfg.as_mut_bytes()[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        writer.write_all(self.cfg.as_bytes())?;

        Ok(())
    }

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        reader.read_exact(self.cfg.as_mut_bytes())?;

        Ok(())
    }
}

impl VirtioPciDevice for VirtioBlkDevice {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioBlkConfig>();
    const CLASS_CODE: u32 = 0x018000;
    const IRQ_PIN: u8 = InterruptPin::INTA as u8;
}
