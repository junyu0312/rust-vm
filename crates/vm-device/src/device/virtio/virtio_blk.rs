use std::sync::Arc;
use std::sync::Mutex;

use vm_core::irq::InterruptController;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_virtio::device::VirtIoDevice;
use vm_virtio::device::blk::config::VirtioBlkConfig;
use vm_virtio::device::blk::req::VirtIoBlkReqType;
use vm_virtio::device::blk::req::VirtioBlkReq;
use vm_virtio::device::pci::VirtIoPciDevice;
use vm_virtio::result::Result;
use vm_virtio::transport::mmio::VirtIoMmioTransport;
use vm_virtio::types::device_features::VIRTIO_F_VERSION_1;
use vm_virtio::types::device_id::DeviceId;
use vm_virtio::types::interrupt_status::InterruptStatus;
use vm_virtio::virt_queue::VirtQueue;
use zerocopy::IntoBytes;

pub struct VirtIoBlkDevice<C> {
    irq: u32,
    irq_chip: Arc<dyn InterruptController>,
    mm: Arc<Mutex<MemoryAddressSpace<C>>>,
    cfg: VirtioBlkConfig,
}

impl<C> VirtIoBlkDevice<C>
where
    C: MemoryContainer,
{
    pub fn new(
        irq: u32,
        irq_chip: Arc<dyn InterruptController>,
        mm: Arc<Mutex<MemoryAddressSpace<C>>>,
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

impl<C> VirtIoDevice for VirtIoBlkDevice<C>
where
    C: MemoryContainer,
{
    const NAME: &str = "virtio-blk";
    const DEVICE_ID: u32 = DeviceId::Blk as u32;
    const VIRT_QUEUES_SIZE_MAX: &[u32] = &[512];
    const DEVICE_FEATURES: u64 = (1 << VIRTIO_F_VERSION_1);

    fn irq(&self) -> Option<u32> {
        Some(self.irq)
    }

    fn trigger_irq(&self, active: bool) {
        self.irq_chip.trigger_irq(32 + self.irq, active);
    }

    fn reset(&mut self) {}

    fn queue_notify(&mut self, virt_queues: &mut [VirtQueue], val: u32) -> Option<InterruptStatus> {
        let queue_sel = val as usize;
        assert_eq!(queue_sel, 0);

        // TODO: async
        let mut mm = self.mm.lock().unwrap();

        let q = &mut virt_queues[queue_sel];

        let avail_ring = q.avail_ring(&mut mm).unwrap();
        let desc_ring = q.desc_table_ref(&mut mm).unwrap();
        let mut used_ring = q.used_ring(&mut mm).unwrap();

        while q.last_available_idx() != avail_ring.idx() {
            let last_available_idx = q.last_available_idx();
            let desc_id = avail_ring.ring(last_available_idx);
            let desc_entry = desc_ring.get(desc_id);
            let req = desc_entry.addr(&mut mm).unwrap();
            let req = unsafe { &*(req.as_ptr() as *const VirtioBlkReq) };

            match req.r#type {
                VirtIoBlkReqType::VirtioBlkTIn => {
                    let chains = desc_ring.get_chain(desc_id);

                    let data = chains[1];
                    let data_hva = data.addr(&mut mm).unwrap();
                    let data_len = data.len;
                    unsafe { data_hva.write_bytes(0xff, data_len.try_into().unwrap()) };

                    let status = chains[2];
                    let mut status_hva = status.addr(&mut mm).unwrap();
                    *unsafe { status_hva.as_mut() } = 0;

                    let used_idx = used_ring.idx();
                    let used_entry = used_ring.ring(used_idx);
                    used_entry.id = desc_id as u32;
                    used_entry.len = data_len; // TODO: +1 for status?
                    used_ring.incr_idx();
                }
                VirtIoBlkReqType::VirtioBlkTOut => todo!(),
                VirtIoBlkReqType::VirtioBlkTFlush => todo!(),
                VirtIoBlkReqType::VirtioBlkTGetId => todo!(),
                VirtIoBlkReqType::VirtioBlkTGetLifetime => todo!(),
                VirtIoBlkReqType::VirtioBlkTDiscard => todo!(),
                VirtIoBlkReqType::VirtioBlkTWriteZeroes => todo!(),
                VirtIoBlkReqType::VirtioBlkTSecureErase => todo!(),
            }

            q.incr_last_available_idx();
        }

        Some(InterruptStatus::VIRTIO_MMIO_INT_VRING)
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

impl<C> VirtIoPciDevice for VirtIoBlkDevice<C>
where
    C: MemoryContainer,
{
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = size_of::<VirtioBlkConfig>();
    const CLASS_CODE: u32 = 0x018000;
}

pub type VirtIoMmioBlkDevice<C> = VirtIoMmioTransport<VirtIoBlkDevice<C>>;
