use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::thread;

use lazy_static::lazy_static;
use tracing::warn;
use vm_core::device::Device;
use vm_core::device::mmio::MmioDevice;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;

use crate::virtio::device::virtio_input::VIRTIO_INPUT_EVENTS_Q;
use crate::virtio::device::virtio_input::VIRTIO_INPUT_VIRT_QUEUE;
use crate::virtio::device::virtio_input::VirtIOInput;
use crate::virtio::device::virtio_input::VirtioInputConfig;
use crate::virtio::device::virtio_input::linux_evdev::EventTypes;
use crate::virtio::device::virtio_input::linux_evdev::ev_key::EvKeyCode;
use crate::virtio::device::virtio_input::linux_evdev::ev_key::KeyValue;
use crate::virtio::device::virtio_input::linux_evdev::ev_syn::EvSynCode;
use crate::virtio::device::virtio_input::virtio_input_event::VirtioInputEvent;
use crate::virtio::transport::VirtIo;
use crate::virtio::transport::mmio::VirtIoMmio;
use crate::virtio::transport::mmio::VirtIoMmioAdaptor;
use crate::virtio::types::device_features::DeviceFeatures;
use crate::virtio::types::device_features::VIRTIO_F_VERSION_1;
use crate::virtio::types::driver_features::DriverFeatures;
use crate::virtio::types::interrupt::InterruptStatus;
use crate::virtio::types::status::Status;
use crate::virtio::types::virtqueue::split_virtqueue::VirtQueue;

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

pub struct VirtIOMmioKbd<const IRQ: u32, C: MemoryContainer>(
    Arc<Mutex<VirtIoMmioAdaptor<VirtIOMmioKbdInternal<IRQ, C>>>>,
);

impl<const IRQ: u32, C> VirtIOMmioKbd<IRQ, C>
where
    C: MemoryContainer,
{
    pub fn new(
        mm: Arc<Mutex<MemoryAddressSpace<C>>>,
        serial: String,
        mmio_range: MmioRange,
        irq_chip: Arc<dyn InterruptController>,
        rx: Receiver<u8>,
    ) -> Self {
        let inner = Arc::new(Mutex::new(VirtIoMmioAdaptor::from(VirtIOMmioKbdInternal {
            mm,
            irq_chip,
            mmio_range,

            device_features: DeviceFeatures::from_u64(DEVICE_FEATURE),
            device_feature_sel: Default::default(),
            driver_features: Default::default(),
            driver_feature_sel: Default::default(),
            queue_sel: Default::default(),
            virtqueues: Default::default(),
            interrupt_status: Default::default(),
            status: Default::default(),
            config_generation: Default::default(),
            input_config: Default::default(),

            serial,

            last_avail_idx: Default::default(),
        })));

        thread::spawn({
            let inner = inner.clone();
            move || {
                while let Ok(byte) = rx.recv() {
                    let mut inner = inner.lock().unwrap();
                    inner.as_mut().trigger_kbd_event(byte).unwrap();
                }
            }
        });

        VirtIOMmioKbd(inner)
    }
}

impl<const IRQ: u32, C> Device for VirtIOMmioKbd<IRQ, C>
where
    C: MemoryContainer,
{
    fn name(&self) -> String {
        let inner = self.0.lock().unwrap();
        inner.name()
    }

    fn as_mmio_device(&self) -> Option<&dyn MmioDevice> {
        Some(self)
    }

    fn as_mmio_device_mut(&mut self) -> Option<&mut dyn MmioDevice> {
        Some(self)
    }
}

impl<const IRQ: u32, C> MmioDevice for VirtIOMmioKbd<IRQ, C>
where
    C: MemoryContainer,
{
    fn mmio_range(&self) -> MmioRange {
        let inner = self.0.lock().unwrap();

        inner.mmio_range()
    }

    fn mmio_read(&mut self, offset: u64, len: usize, data: &mut [u8]) {
        let mut inner = self.0.lock().unwrap();

        inner.mmio_read(offset, len, data)
    }

    fn mmio_write(&mut self, offset: u64, len: usize, data: &[u8]) {
        let mut inner = self.0.lock().unwrap();

        inner.mmio_write(offset, len, data)
    }

    fn generate_dt(&self, fdt: &mut vm_fdt::FdtWriter) -> Result<(), vm_fdt::Error> {
        let inner = self.0.lock().unwrap();

        inner.generate_dt(fdt)
    }
}

struct VirtIOMmioKbdInternal<const IRQ: u32, C: MemoryContainer> {
    mm: Arc<Mutex<MemoryAddressSpace<C>>>,
    irq_chip: Arc<dyn InterruptController>,
    mmio_range: MmioRange,

    device_features: DeviceFeatures,
    device_feature_sel: Option<u32>,
    driver_features: DriverFeatures,
    driver_feature_sel: Option<u32>,
    queue_sel: Option<u32>,
    virtqueues: [VirtQueue<QUEUE_SIZE_MAX>; VIRTIO_INPUT_VIRT_QUEUE as usize],
    interrupt_status: InterruptStatus,
    status: Status,
    config_generation: u32,
    input_config: VirtioInputConfig,

    last_avail_idx: u16,

    serial: String,
}

impl<const IRQ: u32, C> VirtIOMmioKbdInternal<IRQ, C>
where
    C: MemoryContainer,
{
    fn read_queue_sel_or_zero(&self) -> usize {
        self.queue_sel.unwrap_or_else(|| {
            warn!(
                name = Self::NAME,
                "read queue_sel but sel unset, use 0 as default"
            );
            0
        }) as usize
    }

    fn send_event(&mut self, r#type: EventTypes, code: u16, value: u32) -> anyhow::Result<()> {
        let mut mm = self.mm.lock().unwrap();

        let eventq = &mut self.virtqueues[VIRTIO_INPUT_EVENTS_Q];

        let mut desc_table = eventq.desc_table_ref(&mut mm)?;
        let avail_ring = eventq.avail_ring(&mut mm)?;
        let mut used_ring = eventq.used_ring(&mut mm)?;

        if self.last_avail_idx == avail_ring.idx() {
            // full, ignore the event
            return Ok(());
        }

        println!("send");

        let desc_id = avail_ring.ring(self.last_avail_idx);
        self.last_avail_idx += 1;
        self.last_avail_idx %= eventq.read_queue_size();

        // Fill desc
        {
            let desc = desc_table.get_mut(desc_id);
            assert_eq!(size_of::<VirtioInputEvent>(), desc.len as usize);
            let virtio_input_event_gpa = desc.addr;
            let hva = mm.gpa_to_hva(virtio_input_event_gpa)? as *mut VirtioInputEvent;
            let virtio_input_event = unsafe { &mut *hva };
            virtio_input_event.r#type = r#type as u16;
            virtio_input_event.code = code;
            virtio_input_event.value = value;
        }

        // Fill used
        {
            let used = used_ring.ring(used_ring.idx());
            used.id = desc_id as u32;
            used.len = size_of::<VirtioInputEvent>() as u32;
            used_ring.incr_idx();
        }

        Ok(())
    }

    fn trigger_kbd_event(&mut self, e: u8) -> anyhow::Result<()> {
        if !self.read_status().contains(Status::DRIVER_OK) {
            return Ok(());
        }

        let code = match e {
            13 => EvKeyCode::Enter,
            _ => todo!(),
        };

        self.send_event(r#EventTypes::Key, code as u16, KeyValue::KeyPress as u32)?;
        self.send_event(r#EventTypes::Syn, EvSynCode::Report as u16, 0)?;
        self.send_event(r#EventTypes::Key, code as u16, KeyValue::KeyRelease as u32)?;
        self.send_event(r#EventTypes::Syn, EvSynCode::Report as u16, 0)?;

        self.interrupt_status
            .insert(InterruptStatus::VIRTIO_MMIO_INT_VRING);
        self.irq_chip.trigger_irq(80, true);

        Ok(())
    }
}

impl<const IRQ: u32, C> VirtIOInput for VirtIOMmioKbdInternal<IRQ, C>
where
    C: MemoryContainer,
{
    const INPUT_PROP: u32 = 0;

    fn id_name(&self) -> &str {
        Self::NAME
    }

    fn serial(&self) -> &str {
        &self.serial
    }

    fn bitmap_of_ev(&self, ev: EventTypes) -> Option<&[u8]> {
        match ev {
            EventTypes::Key => Some(KEY_BITMAP.as_ref()),
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

impl<const IRQ: u32, C> VirtIoMmio for VirtIOMmioKbdInternal<IRQ, C>
where
    C: MemoryContainer,
{
    fn mmio_range(&self) -> MmioRange {
        self.mmio_range
    }

    fn interrupts(&self) -> Option<&[u32]> {
        Some(&[0, IRQ, 4])
    }
}

impl<const IRQ: u32, C> VirtIo for VirtIOMmioKbdInternal<IRQ, C>
where
    C: MemoryContainer,
{
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
        self.last_avail_idx = 0;
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

    fn write_queue_size(&mut self, size: u16) {
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

    fn write_queue_notify(&mut self, queue_id: u32) {
        warn!(queue_id, "ignored queue_notify");
    }

    fn read_interrupt_status(&self) -> u32 {
        self.interrupt_status.bits()
    }

    fn write_interrupt_ack(&mut self, val: u32) {
        self.interrupt_status
            .remove(InterruptStatus::from_bits_truncate(val));

        if self.interrupt_status.is_empty() {
            self.irq_chip.trigger_irq(80, false);
        }
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

    fn write_queue_driver_low(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_available_low(addr);
    }

    fn write_queue_driver_high(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_available_high(addr);
    }

    fn write_queue_device_low(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_used_low(addr);
    }

    fn write_queue_device_high(&mut self, addr: u32) {
        let sel = self.read_queue_sel_or_zero();

        self.virtqueues[sel].write_queue_used_high(addr);
    }

    fn read_config_generation(&self) -> u32 {
        self.config_generation
    }
}
