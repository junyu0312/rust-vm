use std::sync::Arc;
use std::sync::Mutex;

use bitflags::Flags;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::warn;

use crate::device::VirtIoDevice;
use crate::result::Result;
use crate::transport::control_register::ControlRegister;
use crate::types::interrupt_status::InterruptStatus;
use crate::types::status::Status;
use crate::virt_queue::VirtQueue;

pub mod control_register;
pub mod mmio;
pub mod pci;

pub struct VirtIoTransport<D> {
    device: D,

    device_feature_sel: Option<u32>,
    driver_features: u64,
    driver_feature_sel: Option<u32>,
    queue_sel: Option<u32>,
    virtqueues: Vec<Arc<Mutex<VirtQueue>>>,
    virtqueue_notifiers: Vec<Arc<Notify>>,
    _virtqueue_workers: Vec<JoinHandle<()>>,
    interrupt_status: Arc<Mutex<InterruptStatus>>,
    status: Status,
    config_generation: u32,
}

impl<D> From<D> for VirtIoTransport<D>
where
    D: VirtIoDevice,
{
    fn from(device: D) -> Self {
        let mut virtqueues = Vec::with_capacity(D::VIRT_QUEUES_SIZE_MAX.len());
        let mut virtqueue_notifiers = Vec::with_capacity(D::VIRT_QUEUES_SIZE_MAX.len());
        let mut virtqueue_workers = Vec::with_capacity(D::VIRT_QUEUES_SIZE_MAX.len());

        let interrupt_status = Arc::new(Mutex::new(Default::default()));

        for (queue, &size_max) in D::VIRT_QUEUES_SIZE_MAX.iter().enumerate() {
            let notify = Arc::new(Notify::new());
            let virtqueue = Arc::new(Mutex::new(VirtQueue::new(size_max)));
            let fut = device
                .virtqueue_handler(
                    queue,
                    notify.clone(),
                    interrupt_status.clone(),
                    virtqueue.clone(),
                )
                .unwrap();

            virtqueues.push(virtqueue);
            virtqueue_notifiers.push(notify);
            virtqueue_workers.push(tokio::spawn(fut));
        }

        VirtIoTransport {
            device,
            device_feature_sel: Default::default(),
            driver_features: Default::default(),
            driver_feature_sel: Default::default(),
            queue_sel: Default::default(),
            virtqueues,
            virtqueue_notifiers,
            _virtqueue_workers: virtqueue_workers,
            interrupt_status,
            status: Default::default(),
            config_generation: Default::default(),
        }
    }
}

impl<D> VirtIoTransport<D>
where
    D: VirtIoDevice,
{
    fn reset(&mut self) {
        self.device.reset();
        self.device_feature_sel = Default::default();
        self.driver_features = Default::default();
        self.driver_feature_sel = Default::default();
        self.queue_sel = Default::default();
        for virtqueue in &self.virtqueues {
            let mut q = virtqueue.lock().unwrap();
            q.reset();
        }
        self.interrupt_status.lock().unwrap().clear();
        self.status = Default::default();
        self.config_generation = Default::default();
    }

    fn get_device_feature_sel_or_default(&self) -> u32 {
        if let Some(sel) = self.device_feature_sel {
            sel
        } else {
            warn!("device_feature_sel unset");
            0
        }
    }

    fn get_driver_feature_sel_or_default(&self) -> u32 {
        if let Some(sel) = self.driver_feature_sel {
            sel
        } else {
            warn!("driver_feature_sel unset");
            0
        }
    }

    fn get_queue_sel_or_default(&self) -> u32 {
        if let Some(sel) = self.queue_sel {
            sel
        } else {
            warn!("queue_sel unset");
            0
        }
    }

    pub fn read_reg(&self, reg: ControlRegister) -> u32 {
        match reg {
            ControlRegister::DeviceFeatures => {
                let sel = self.get_device_feature_sel_or_default();
                // if sel >= 2, just return 0
                (D::DEVICE_FEATURES >> (sel * 32)) as u32
            }
            ControlRegister::DeviceFeaturesSel => self.get_device_feature_sel_or_default(),
            ControlRegister::DriverFeatures => todo!(),
            ControlRegister::DriverFeaturesSel => self.get_driver_feature_sel_or_default(),
            ControlRegister::QueueSel => todo!(),
            ControlRegister::QueueSizeMax => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .read_queue_size_max()
            }
            ControlRegister::QueueSize => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .read_queue_size() as u32
            }
            ControlRegister::QueueReady => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .read_queue_ready() as u32
            }
            ControlRegister::QueueNotify => todo!(),
            ControlRegister::InterruptStatus => self.interrupt_status.lock().unwrap().bits(),
            ControlRegister::Status => self.status.bits() as u32,
            ControlRegister::QueueDescLow => todo!(),
            ControlRegister::QueueDescHigh => todo!(),
            ControlRegister::QueueAvailLow => todo!(),
            ControlRegister::QueueAvailHigh => todo!(),
            ControlRegister::QueueUsedLow => todo!(),
            ControlRegister::QueueUsedHigh => todo!(),
            ControlRegister::ShmSel => todo!(),
            ControlRegister::ShmLenLow => todo!(),
            ControlRegister::ShmLenHigh => todo!(),
            ControlRegister::ShmBaseLow => todo!(),
            ControlRegister::ShmBaseHigh => todo!(),
            ControlRegister::QueueReset => todo!(),
            ControlRegister::ConfigGeneration => self.config_generation,
        }
    }

    pub fn write_reg(&mut self, reg: ControlRegister, val: u32) -> Result<()> {
        match reg {
            ControlRegister::DeviceFeatures => {
                warn!(?reg, "try to write a RO register");
                panic!()
            }
            ControlRegister::DeviceFeaturesSel => self.device_feature_sel = Some(val),
            ControlRegister::DriverFeatures => {
                let sel = self.get_driver_feature_sel_or_default();

                if sel == 0 {
                    self.driver_features =
                        (self.driver_features & 0xffff_ffff_0000_0000) | (val as u64);
                } else if sel == 1 {
                    self.driver_features =
                        (self.driver_features & 0x0000_0000_ffff_ffff) | ((val as u64) << 32);
                } else {
                    assert_eq!(val, 0);
                }
            }
            ControlRegister::DriverFeaturesSel => self.driver_feature_sel = Some(val),
            ControlRegister::QueueSel => self.queue_sel = Some(val),
            ControlRegister::QueueSizeMax => todo!(),
            ControlRegister::QueueSize => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_size(val.try_into().unwrap());
            }
            ControlRegister::QueueReady => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_ready(val != 0);
            }
            ControlRegister::QueueNotify => {
                let queue_sel = val; // since VIRTIO_F_NOTIFICATION_DATA is not enabled
                self.virtqueue_notifiers[queue_sel as usize].notify_one();
            }
            ControlRegister::InterruptStatus => {
                *self.interrupt_status.lock().unwrap() = InterruptStatus::from_bits_truncate(val)
            }
            ControlRegister::Status => {
                if val == 0 {
                    self.reset();
                } else {
                    self.status = Status::from_bits_truncate(val as u8);
                }
            }
            ControlRegister::QueueDescLow => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_desc_low(val);
            }
            ControlRegister::QueueDescHigh => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_desc_high(val);
            }
            ControlRegister::QueueAvailLow => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_available_low(val);
            }
            ControlRegister::QueueAvailHigh => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_available_high(val);
            }
            ControlRegister::QueueUsedLow => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_used_low(val);
            }
            ControlRegister::QueueUsedHigh => {
                let sel = self.get_queue_sel_or_default();
                self.virtqueues[sel as usize]
                    .lock()
                    .unwrap()
                    .write_queue_used_high(val);
            }
            ControlRegister::ShmSel => todo!(),
            ControlRegister::ShmLenLow => todo!(),
            ControlRegister::ShmLenHigh => todo!(),
            ControlRegister::ShmBaseLow => todo!(),
            ControlRegister::ShmBaseHigh => todo!(),
            ControlRegister::QueueReset => todo!(),
            ControlRegister::ConfigGeneration => todo!(),
        }

        Ok(())
    }

    pub fn read_config(&self, offset: usize, len: usize, buf: &mut [u8]) -> Result<()> {
        self.device.read_config(offset, len, buf)
    }

    pub fn write_config(&mut self, offset: usize, len: usize, buf: &[u8]) -> Result<()> {
        self.device.write_config(offset, len, buf)
    }
}
