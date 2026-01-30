use std::sync::Arc;

use lazy_static::lazy_static;
use tracing::warn;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;

use crate::virtio::device::virtio_input::Ev;
use crate::virtio::device::virtio_input::VIRTIO_INPUT_VIRT_QUEUE;
use crate::virtio::device::virtio_input::VirtIOInput;
use crate::virtio::device::virtio_input::VirtioInputConfig;
use crate::virtio::transport::VirtIo;
use crate::virtio::transport::mmio::VirtIoMmio;
use crate::virtio::types::device_features::DeviceFeatures;
use crate::virtio::types::device_features::VIRTIO_F_VERSION_1;
use crate::virtio::types::driver_features::DriverFeatures;
use crate::virtio::types::status::Status;
use crate::virtio::types::virtqueue::VirtQueue;

fn bit_to_index(bit: usize) -> (usize, usize) {
    (bit / 8, bit % 8)
}

lazy_static! {
     static ref KEY_BITMAP: [u8; 32] = {
        let mut bitmap = [0u8; 32];

        // TODO: More
        let enter_bit: usize = 28;
        let (byte_index, bit_index) = bit_to_index(enter_bit);

        bitmap[byte_index] |= 1 << bit_index;
        bitmap
    };
}

const DEVICE_FEATURE: u64 = 1 << VIRTIO_F_VERSION_1;
const QUEUE_SIZE_MAX: u32 = 256;

pub struct VirtIOMmioKbd<const IRQ: u32> {
    mmio_range: MmioRange,
    _irq_chip: Arc<dyn InterruptController>,
    serial: String,

    device_features: DeviceFeatures,
    device_feature_sel: Option<u32>,
    driver_features: DriverFeatures,
    driver_feature_sel: Option<u32>,
    queue_sel: Option<u32>,
    virtqueues: [VirtQueue<QUEUE_SIZE_MAX>; VIRTIO_INPUT_VIRT_QUEUE as usize],
    status: Status,
    config_generation: u32,
    input_config: VirtioInputConfig,
}

impl<const IRQ: u32> VirtIOMmioKbd<IRQ> {
    pub fn new(
        serial: String,
        mmio_range: MmioRange,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Self {
        VirtIOMmioKbd {
            mmio_range,
            _irq_chip: irq_chip,
            serial,

            device_features: DeviceFeatures::from_u64(DEVICE_FEATURE),
            device_feature_sel: Default::default(),
            driver_features: Default::default(),
            driver_feature_sel: Default::default(),
            queue_sel: Default::default(),
            virtqueues: Default::default(),
            status: Default::default(),
            config_generation: Default::default(),
            input_config: Default::default(),
        }
    }

    fn read_queue_sel_or_zero(&self) -> usize {
        self.queue_sel.unwrap_or_else(|| {
            warn!(
                name = Self::NAME,
                "read queue_sel but sel unset, use 0 as default"
            );
            0
        }) as usize
    }
}

impl<const IRQ: u32> VirtIOInput for VirtIOMmioKbd<IRQ> {
    const INPUT_PROP: u32 = 0;

    fn id_name(&self) -> &str {
        Self::NAME
    }

    fn serial(&self) -> &str {
        &self.serial
    }

    fn bitmap_of_ev(&self, ev: Ev) -> Option<&[u8]> {
        match ev {
            Ev::Key => Some(KEY_BITMAP.as_ref()),
            _ => None,
        }
    }

    fn get_virtio_input_config(&self) -> &VirtioInputConfig {
        &self.input_config
    }

    fn get_virtio_input_config_mut(&mut self) -> &mut VirtioInputConfig {
        &mut self.input_config
    }
}

impl<const IRQ: u32> VirtIoMmio for VirtIOMmioKbd<IRQ> {
    fn mmio_range(&self) -> &MmioRange {
        &self.mmio_range
    }

    fn interrupts(&self) -> Option<&[u32]> {
        Some(&[0, IRQ, 4])
    }
}

impl<const IRQ: u32> VirtIo for VirtIOMmioKbd<IRQ> {
    type Subsystem = Self;

    const NAME: &str = "virtio-mmio-kbd";
    const VIRT_QUEUES: u32 = VIRTIO_INPUT_VIRT_QUEUE;

    fn reset(&mut self) {
        // device_feature never change
        self.device_feature_sel = None;
        self.driver_features = Default::default();
        self.driver_feature_sel = None;
        self.queue_sel = None;
        self.virtqueues = Default::default();
        self.status = Status::default();
        self.config_generation = Default::default();
        self.input_config = VirtioInputConfig::default();
    }

    fn read_device_features(&self) -> u32 {
        let sel = self.device_feature_sel.unwrap_or_else(|| {
            warn!(
                name = Self::NAME,
                "read device_feature but sel unset, use 0 as default"
            );
            0
        });

        self.device_features.read(sel as usize)
    }

    fn write_device_feature_sel(&mut self, sel: u32) {
        self.device_feature_sel = Some(sel);
    }

    fn write_driver_features(&mut self, feat: u32) {
        let sel = self.driver_feature_sel.unwrap_or_else(|| {
            warn!(
                name = Self::NAME,
                "write driver_feature but sel unset, use 0 as default"
            );
            0
        });

        self.driver_features.write(sel as usize, feat);
    }

    fn write_driver_feature_sel(&mut self, sel: u32) {
        self.driver_feature_sel = Some(sel);
    }

    fn write_queue_sel(&mut self, sel: u32) {
        self.queue_sel = Some(sel);
    }

    fn read_queue_size_max(&self) -> u32 {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].queue_size_max()
    }

    fn write_queue_size(&mut self, size: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_size(size);
    }

    fn read_queue_ready(&self) -> bool {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].read_queue_ready()
    }

    fn write_queue_ready(&mut self, queue_ready: bool) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_ready(queue_ready)
    }

    fn read_status(&self) -> Status {
        self.status
    }

    fn write_status_non_zero(&mut self, status: Status) {
        self.status = status;

        if self.status.device_needs_reset() || self.status.failed() {
            todo!()
        }
    }

    fn write_queue_desc_low(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_desc_low(addr);
    }

    fn write_queue_desc_high(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_desc_high(addr);
    }

    fn write_queue_avail_low(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_avail_low(addr);
    }

    fn write_queue_avail_high(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_avail_high(addr);
    }

    fn write_queue_used_low(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_used_low(addr);
    }

    fn write_queue_used_high(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_used_high(addr);
    }

    fn read_config_generation(&self) -> u32 {
        self.config_generation
    }
}
