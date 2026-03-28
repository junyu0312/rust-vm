use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_virtio::device::VirtioDevice;
use vm_virtio::device::virtqueue::VirtqueueHandler;
use vm_virtio::device::virtqueue::VirtqueueHandlerFn;
use vm_virtio::result::Result;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::mmio::VirtioMmioTransport;
use vm_virtio::transport::pci::VirtioPciDevice;
use vm_virtio::types::device::blk::config::VirtioBlkConfig;
use vm_virtio::types::device::blk::req::VirtioBlkReq;
use vm_virtio::types::device::blk::req::VirtioBlkReqType;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use zerocopy::IntoBytes;

fn requestq0_handler<D>() -> VirtqueueHandlerFn<D> {
    Box::new(|mm, _dev, desc_ring, desc_id| {
        let desc_entry = desc_ring.get(desc_id);
        let req = desc_entry.addr(mm).unwrap();
        let req = unsafe { &*(req.as_ptr() as *const VirtioBlkReq) };

        match req.r#type {
            VirtioBlkReqType::VirtioBlkTIn => {
                let chains = desc_ring.get_chain(desc_id);

                let data = chains[1];
                let data_hva = data.addr(mm).unwrap();
                let data_len = data.len;
                unsafe { data_hva.write_bytes(0xff, data_len.try_into().unwrap()) };

                let status = chains[2];
                let mut status_hva = status.addr(mm).unwrap();
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
    })
}

pub struct VirtioBlkDevice {
    irq: u32,
    irq_chip: Arc<dyn InterruptController>,
    mm: Arc<MemoryAddressSpace>,
    cfg: VirtioBlkConfig,
}

impl VirtioBlkDevice {
    pub fn new(
        irq: u32,
        irq_chip: Arc<dyn InterruptController>,
        mm: Arc<MemoryAddressSpace>,
    ) -> Self {
        let cfg = VirtioBlkConfig {
            capacity: 50,
            ..Default::default()
        };

        VirtioBlkDevice {
            irq,
            irq_chip,
            mm,
            cfg,
        }
    }
}

impl VirtioDevice for VirtioBlkDevice {
    const NAME: &str = "virtio-blk";
    const DEVICE_ID: u16 = DeviceId::Blk as u16;
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn virtqueues_size_max(&self) -> Vec<Option<u32>> {
        vec![Some(512)]
    }

    fn irq(&self) -> Option<u32> {
        Some(self.irq)
    }

    fn trigger_irq(&self, active: bool) {
        self.irq_chip.trigger_irq(32 + self.irq, active);
    }

    fn reset(&mut self) {}

    fn virtqueue_handler(
        &self,
        queue_sel: usize,
        notifier: Arc<Notify>,
        dev: Arc<Mutex<VirtioDev<Self>>>,
    ) -> Option<VirtqueueHandler<Self>> {
        if queue_sel != 0 {
            return None;
        }

        Some(VirtqueueHandler {
            queue_sel,
            notifier,
            dev,
            mm: self.mm.clone(),
            irq_chip: self.irq_chip.clone(),
            irq_line: 32 + self.irq,
            handle_desc: requestq0_handler(),
        })
    }

    fn read_config(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, buf: &[u8]) -> Result<()> {
        self.cfg.as_mut_bytes()[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

impl VirtioPciDevice for VirtioBlkDevice {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioBlkConfig>();
    const CLASS_CODE: u32 = 0x018000;
    const IRQ_PIN: u8 = InterruptPin::INTA as u8;
}

pub type VirtioMmioBlkDevice = VirtioMmioTransport<VirtioBlkDevice>;
