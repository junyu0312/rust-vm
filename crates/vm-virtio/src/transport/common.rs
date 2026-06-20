use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::task::JoinHandle;
use vm_core::device::error::DeviceSnapshotError;
use vm_snapshot::helper::read_u8;
use vm_snapshot::helper::read_u16;
use vm_snapshot::helper::read_u32;
use vm_snapshot::helper::read_u64;
use vm_snapshot::helper::write_u8;
use vm_snapshot::helper::write_u16;
use vm_snapshot::helper::write_u32;

use crate::device::VirtioDevice;
use crate::device::virtqueue::VirtqueueWorkerController;
use crate::result::Result;
use crate::result::VirtioError;
use crate::transport::common::control_register::ControlRegister;
use crate::types::interrupt_status::InterruptStatus;
use crate::types::status::Status;
use crate::virtqueue::Virtqueue;

pub(crate) mod control_register;

const DEVICE_FEATURE_SEL_MAX: u32 = 1;
const DRIVER_FEATURE_SEL_MAX: u32 = 1;

pub struct VirtqueueHandler {
    pub controller: Arc<VirtqueueWorkerController>,
    pub _join_handler: JoinHandle<()>,
}

/// Common state for a VirtIO transport implementation.
pub struct VirtioTransportCommon<D> {
    pub device: D,

    // common registers
    device_feature_sel: u32,
    driver_features: u64,
    driver_feature_sel: u32,
    queue_sel: u16,
    virtqueues: Vec<Virtqueue>,
    interrupt_status: Arc<Mutex<InterruptStatus>>,
    status: Status,
    config_generation: Arc<Mutex<u8>>,
}

impl<D> VirtioTransportCommon<D>
where
    D: VirtioDevice,
{
    pub fn new(device: D) -> Result<Self> {
        let virtqueues_size_max = device.virtqueues_size_max();

        let mut virtqueues = Vec::with_capacity(virtqueues_size_max.len());
        for queue_size_max in &virtqueues_size_max {
            virtqueues.push(Virtqueue::new(*queue_size_max));
        }

        let virtio_dev = VirtioTransportCommon {
            device,

            device_feature_sel: Default::default(),
            driver_features: Default::default(),
            driver_feature_sel: Default::default(),
            queue_sel: Default::default(),
            virtqueues,
            interrupt_status: Default::default(),
            status: Default::default(),
            config_generation: Default::default(),
        };

        Ok(virtio_dev)
    }
}

impl<D> VirtioTransportCommon<D>
where
    D: VirtioDevice,
{
    pub fn reset(&mut self) {
        self.device.reset();
        self.device_feature_sel = Default::default();
        self.driver_features = Default::default();
        self.driver_feature_sel = Default::default();
        self.queue_sel = Default::default();
        for virtqueue in self.virtqueues.iter_mut() {
            virtqueue.reset();
        }
        *self.interrupt_status.lock().unwrap() = InterruptStatus::empty();
        self.status = Default::default();
        *self.config_generation.lock().unwrap() = 0;
    }

    fn get_device_feature_sel(&self) -> u32 {
        self.device_feature_sel
    }

    fn get_driver_feature_sel(&self) -> u32 {
        self.driver_feature_sel
    }

    pub fn get_queue_sel(&self) -> u16 {
        self.queue_sel
    }

    pub fn read_reg(&self, reg: ControlRegister) -> Result<u32> {
        let val = match reg {
            ControlRegister::DeviceFeatures => {
                let sel = self.get_device_feature_sel();
                // The condition is not necessary, but we keep it for readable
                if sel > DEVICE_FEATURE_SEL_MAX {
                    // This means that it will present 0 for any device_feature_select other than 0 or 1, since no feature
                    // defined here exceeds 63.
                    0
                } else {
                    (D::DEVICE_FEATURES >> (sel * 32)) as u32
                }
            }
            ControlRegister::DeviceFeaturesSel => self.get_device_feature_sel(),
            ControlRegister::DriverFeatures => {
                let sel = self.get_driver_feature_sel();
                // The condition is not necessary, but we keep it for readable
                if sel > DRIVER_FEATURE_SEL_MAX {
                    // This means that it will present 0 for any device_feature_select other than 0 or 1, since no feature
                    // defined here exceeds 63.
                    0
                } else {
                    (self.driver_features >> (sel * 32)) as u32
                }
            }
            ControlRegister::DriverFeaturesSel => self.get_driver_feature_sel(),
            ControlRegister::QueueSel => todo!(),
            ControlRegister::QueueSizeMax => {
                // Reading from the register returns the maximum size (number of elements)
                // of the queue the device is ready to process or zero (0x0) if the queue is not
                // available. This applies to the queue selected by writing to QueueSel.
                let sel = self.get_queue_sel();
                self.virtqueues
                    .get(sel as usize)
                    .map(|vq| vq.read_queue_size_max() as u32)
                    .unwrap_or(0)
            }
            ControlRegister::QueueSize => {
                // The device MUST present a 0 in queue_size if the virtqueue corresponding to the current queue_select is
                // unavailable.
                let sel = self.get_queue_sel();
                self.virtqueues
                    .get(sel as usize)
                    .map(|vq| vq.read_queue_size() as u32)
                    .unwrap_or(0)
            }
            ControlRegister::QueueReady => {
                let sel = self.get_queue_sel();
                self.get_virtqueue(sel)?.read_queue_ready() as u32
            }
            ControlRegister::InterruptStatus => self.interrupt_status.lock().unwrap().bits(),
            ControlRegister::Status => self.status.bits() as u32,
            ControlRegister::QueueDescLow => unreachable!(),
            ControlRegister::QueueDescHigh => unreachable!(),
            ControlRegister::QueueAvailLow => unreachable!(),
            ControlRegister::QueueAvailHigh => unreachable!(),
            ControlRegister::QueueUsedLow => unreachable!(),
            ControlRegister::QueueUsedHigh => unreachable!(),
            ControlRegister::ShmSel => unreachable!(),
            ControlRegister::ShmLenLow => unreachable!(),
            ControlRegister::ShmLenHigh => unreachable!(),
            ControlRegister::ShmBaseLow => unreachable!(),
            ControlRegister::ShmBaseHigh => unreachable!(),
            ControlRegister::QueueReset => unreachable!(),
            ControlRegister::ConfigGeneration => *self.config_generation.lock().unwrap() as u32,
        };

        Ok(val)
    }

    pub fn write_reg(&mut self, reg: ControlRegister, val: u32) -> Result<()> {
        match reg {
            ControlRegister::DeviceFeatures => unreachable!(),
            ControlRegister::DeviceFeaturesSel => self.device_feature_sel = val,
            ControlRegister::DriverFeatures => {
                let sel = self.get_driver_feature_sel();

                let shift = sel * 32;
                let mask = 0xffff_ffffu64.wrapping_shl(shift);

                let filtered_val = ((val as u64).wrapping_shl(shift)) & D::DEVICE_FEATURES;

                self.driver_features = (self.driver_features & !mask) | filtered_val;
            }
            ControlRegister::DriverFeaturesSel => self.driver_feature_sel = val,
            ControlRegister::QueueSel => {
                self.queue_sel = val
                    .try_into()
                    .map_err(|_| VirtioError::QueueExceedsU16 { device: D::NAME })?
            }
            ControlRegister::QueueSizeMax => todo!(),
            ControlRegister::QueueSize => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?
                    .write_queue_size(val.try_into().unwrap());
            }
            ControlRegister::QueueReady => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_ready(val != 0);
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
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_desc_low(val);
            }
            ControlRegister::QueueDescHigh => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_desc_high(val);
            }
            ControlRegister::QueueAvailLow => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_available_low(val);
            }
            ControlRegister::QueueAvailHigh => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_available_high(val);
            }
            ControlRegister::QueueUsedLow => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_used_low(val);
            }
            ControlRegister::QueueUsedHigh => {
                let sel = self.get_queue_sel();
                self.get_virtqueue_mut(sel)?.write_queue_used_high(val);
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

    pub fn get_virtqueue(&self, queue_sel: u16) -> Result<&Virtqueue> {
        self.virtqueues
            .get(queue_sel as usize)
            .ok_or(VirtioError::VirtqueueNotFound { queue_sel })
    }

    pub fn get_virtqueue_mut(&mut self, queue_sel: u16) -> Result<&mut Virtqueue> {
        self.virtqueues
            .get_mut(queue_sel as usize)
            .ok_or(VirtioError::VirtqueueNotFound { queue_sel })
    }

    pub fn get_interrupt_status(&self) -> Arc<Mutex<InterruptStatus>> {
        self.interrupt_status.clone()
    }

    pub fn get_config_generation(&self) -> Arc<Mutex<u8>> {
        self.config_generation.clone()
    }

    pub fn pause(&self) -> std::result::Result<(), DeviceSnapshotError> {
        todo!()
    }

    pub fn resume(&self) -> std::result::Result<(), DeviceSnapshotError> {
        todo!()
    }

    pub fn save(&self, writer: &mut dyn Write) -> std::result::Result<(), DeviceSnapshotError> {
        self.device.save(writer)?;
        write_u32(writer, self.device_feature_sel)?;
        writer.write_all(&self.driver_features.to_le_bytes())?;
        write_u32(writer, self.driver_feature_sel)?;
        write_u16(writer, self.queue_sel)?;

        for virtqueue in &self.virtqueues {
            virtqueue.save(writer)?;
        }

        write_u32(writer, self.interrupt_status.lock().unwrap().bits())?;
        write_u8(writer, self.status.bits())?;
        write_u8(writer, *self.config_generation.lock().unwrap())?;

        Ok(())
    }

    pub fn load(&mut self, reader: &mut dyn Read) -> std::result::Result<(), DeviceSnapshotError> {
        self.device.load(reader)?;
        self.device_feature_sel = read_u32(reader)?;
        self.driver_features = read_u64(reader)?;
        self.driver_feature_sel = read_u32(reader)?;
        self.queue_sel = read_u16(reader)?;

        for virtqueue in &mut self.virtqueues {
            virtqueue.load(reader)?;
        }

        *self.interrupt_status.lock().unwrap() =
            InterruptStatus::from_bits_retain(read_u32(reader)?);
        self.status = Status::from_bits_retain(read_u8(reader)?);
        *self.config_generation.lock().unwrap() = read_u8(reader)?;

        Ok(())
    }
}
