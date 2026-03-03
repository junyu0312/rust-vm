use std::sync::Arc;

use tokio::sync::Notify;
use vm_core::arch::irq::InterruptController;
use vm_mm::allocator::MemoryContainer;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::interrupt::legacy::InterruptPin;
use vm_virtio::device::VirtIoDevice;
use vm_virtio::device::VirtqueueHandler;
use vm_virtio::device::VirtqueueHandlerFn;
use vm_virtio::device::blk::config::VirtioBlkConfig;
use vm_virtio::device::blk::req::VirtIoBlkReqType;
use vm_virtio::device::blk::req::VirtioBlkReq;
use vm_virtio::device::pci::VirtIoPciDevice;
use vm_virtio::result::Result;
use vm_virtio::transport::VirtIoDev;
use vm_virtio::transport::mmio::VirtIoMmioTransport;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use zerocopy::IntoBytes;

fn requestq0_handler<C, D>() -> VirtqueueHandlerFn<C, D>
where
    C: MemoryContainer,
{
    Box::new(|mm, _dev, desc_ring, desc_id| {
        let desc_entry = desc_ring.get(desc_id);
        let req = desc_entry.addr(mm).unwrap();
        let req = unsafe { &*(req.as_ptr() as *const VirtioBlkReq) };

        match req.r#type {
            VirtIoBlkReqType::VirtioBlkTIn => {
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
            VirtIoBlkReqType::VirtioBlkTOut => todo!(),
            VirtIoBlkReqType::VirtioBlkTFlush => todo!(),
            VirtIoBlkReqType::VirtioBlkTGetId => todo!(),
            VirtIoBlkReqType::VirtioBlkTGetLifetime => todo!(),
            VirtIoBlkReqType::VirtioBlkTDiscard => todo!(),
            VirtIoBlkReqType::VirtioBlkTWriteZeroes => todo!(),
            VirtIoBlkReqType::VirtioBlkTSecureErase => todo!(),
        }
    })
}

pub struct VirtIoBlkDevice<C> {
    irq: u32,
    irq_chip: Arc<dyn InterruptController>,
    mm: Arc<MemoryAddressSpace<C>>,
    cfg: VirtioBlkConfig,
}

impl<C> VirtIoBlkDevice<C>
where
    C: MemoryContainer,
{
    pub fn new(
        irq: u32,
        irq_chip: Arc<dyn InterruptController>,
        mm: Arc<MemoryAddressSpace<C>>,
    ) -> Self {
        let cfg = VirtioBlkConfig {
            capacity: 50,
            ..Default::default()
        };

        VirtIoBlkDevice {
            irq,
            irq_chip,
            mm,
            cfg,
        }
    }
}

impl<C> VirtIoDevice<C> for VirtIoBlkDevice<C>
where
    C: MemoryContainer,
{
    const NAME: &str = "virtio-blk";
    const DEVICE_ID: u32 = DeviceId::Blk as u32;
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
        dev: VirtIoDev<C, Self>,
    ) -> Option<VirtqueueHandler<C, Self>> {
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

    fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()> {
        buf.copy_from_slice(&self.cfg.as_bytes()[offset..offset + len]);
        Ok(())
    }

    fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()> {
        self.cfg.as_mut_bytes()[offset..len].copy_from_slice(buf);
        Ok(())
    }
}

impl<C> VirtIoPciDevice<C> for VirtIoBlkDevice<C>
where
    C: MemoryContainer,
{
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioBlkConfig>();
    const CLASS_CODE: u32 = 0x018000;
    const IRQ_PIN: u8 = InterruptPin::INTA as u8;
}

pub type VirtIoMmioBlkDevice<C> = VirtIoMmioTransport<C, VirtIoBlkDevice<C>>;
