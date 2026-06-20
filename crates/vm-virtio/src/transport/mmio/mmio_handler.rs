use std::sync::Arc;

use tracing::trace;

use crate::device::VirtioDevice;
use crate::device::virtqueue::VirtqueueWorkerController;
use crate::device::virtqueue::virtqueue_worker;
use crate::result::Result;
use crate::result::VirtioError;
use crate::transport::common::control_register::ControlRegister;
use crate::transport::mmio::VirtioMmioTransport;
use crate::transport::mmio::VirtqueueHandler;
use crate::transport::mmio::control_register::MmioControlRegister;
use crate::types::interrupt_status::InterruptStatus;

const CONFIGURATION_SPACE_OFFSET: usize = 0x100;

const VIRTIO_MMIO_MAGIC_VALUE: u32 = u32::from_le_bytes(*b"virt");
const VIRTIO_MMIO_VERSION: u32 = 0x2;
const VIRTIO_MMIO_VENDOR_ID: u32 = u32::from_le_bytes(*b"QEMU");

impl<D> VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    fn read_reg(&self, reg: MmioControlRegister) -> Result<u32> {
        let common = self.common.lock().unwrap();

        match reg {
            MmioControlRegister::MagicValue => Ok(VIRTIO_MMIO_MAGIC_VALUE),
            MmioControlRegister::Version => Ok(VIRTIO_MMIO_VERSION),
            MmioControlRegister::DeviceId => Ok(D::DEVICE_ID as u32),
            MmioControlRegister::VendorId => Ok(VIRTIO_MMIO_VENDOR_ID),
            MmioControlRegister::DeviceFeatures => common.read_reg(ControlRegister::DeviceFeatures),
            MmioControlRegister::QueueSizeMax => common.read_reg(ControlRegister::QueueSizeMax),
            MmioControlRegister::QueueReady => common.read_reg(ControlRegister::QueueReady),
            MmioControlRegister::InterruptStatus => {
                common.read_reg(ControlRegister::InterruptStatus)
            }
            MmioControlRegister::Status => common.read_reg(ControlRegister::Status),
            MmioControlRegister::QueueReset => todo!(),
            MmioControlRegister::ConfigGeneration => {
                common.read_reg(ControlRegister::ConfigGeneration)
            }
            _ => unreachable!("read a wo register"),
        }
    }

    fn write_reg(&self, reg: MmioControlRegister, val: u32) -> Result<()> {
        let mut common = self.common.lock().unwrap();

        match reg {
            MmioControlRegister::DeviceFeaturesSel => {
                common.write_reg(ControlRegister::DeviceFeaturesSel, val)
            }
            MmioControlRegister::DriverFeatures => {
                common.write_reg(ControlRegister::DriverFeatures, val)
            }
            MmioControlRegister::DriverFeaturesSel => {
                common.write_reg(ControlRegister::DriverFeaturesSel, val)
            }
            MmioControlRegister::QueueSel => common.write_reg(ControlRegister::QueueSel, val),
            MmioControlRegister::QueueSize => common.write_reg(ControlRegister::QueueSize, val),
            MmioControlRegister::QueueReady => {
                let mut virtqueue = self.virtqueue_handlers.write().unwrap();
                let queue_sel = common.get_queue_sel();

                if val == 0 {
                    // disable
                    todo!()
                } else {
                    let Some(handler) = common.device.virtqueue_handler(queue_sel) else {
                        unreachable!("no handler for queue {queue_sel}");
                    };

                    let controller = Arc::new(VirtqueueWorkerController::default());

                    let _join_handler = self.tokio_runtime.spawn(virtqueue_worker(
                        self.memory.clone(),
                        controller.clone(),
                        self.get_used_buffer_notification(),
                        *common.get_virtqueue(queue_sel)?,
                        handler,
                    ));

                    assert!(
                        virtqueue
                            .insert(
                                queue_sel,
                                VirtqueueHandler {
                                    controller,
                                    _join_handler,
                                },
                            )
                            .is_none()
                    );
                }

                common.write_reg(ControlRegister::QueueReady, val)
            }
            MmioControlRegister::QueueNotify => {
                let queue_sel = val
                    .try_into()
                    .map_err(|_| VirtioError::QueueExceedsU16 { device: D::NAME })?;
                let handlers = self.virtqueue_handlers.read().unwrap();
                let Some(controller) = handlers.get(&queue_sel) else {
                    unreachable!()
                };
                controller.controller.queue_notify.notify_one();

                Ok(())
            }
            MmioControlRegister::InterruptAck => {
                let is = common.get_interrupt_status();
                let mut is = is.lock().unwrap();

                is.remove(InterruptStatus::from_bits_truncate(val));
                if is.is_empty() {
                    self.irq_chip.trigger_irq(self.irq as u32, false);
                }

                Ok(())
            }
            MmioControlRegister::Status => common.write_reg(ControlRegister::Status, val),
            MmioControlRegister::QueueDescLow => {
                common.write_reg(ControlRegister::QueueDescLow, val)
            }
            MmioControlRegister::QueueDescHigh => {
                common.write_reg(ControlRegister::QueueDescHigh, val)
            }
            MmioControlRegister::QueueAvailLow => {
                common.write_reg(ControlRegister::QueueAvailLow, val)
            }
            MmioControlRegister::QueueAvailHigh => {
                common.write_reg(ControlRegister::QueueAvailHigh, val)
            }
            MmioControlRegister::QueueUsedLow => {
                common.write_reg(ControlRegister::QueueUsedLow, val)
            }
            MmioControlRegister::QueueUsedHigh => {
                common.write_reg(ControlRegister::QueueUsedHigh, val)
            }
            MmioControlRegister::ShmSel => todo!(),
            MmioControlRegister::QueueReset => todo!(),
            _ => unreachable!("write a ro register"),
        }
    }
}

impl<D> VirtioMmioTransport<D>
where
    D: VirtioDevice,
{
    pub fn read(&self, offset: u64, data: &mut [u8]) -> Result<()> {
        if offset < CONFIGURATION_SPACE_OFFSET as u64 {
            let Some(reg) = MmioControlRegister::from_repr(offset as u16) else {
                return Err(VirtioError::MmioReadInvalidRegisterOffset);
            };

            if data.len() == 4 {
                let val = self.read_reg(reg)?;

                trace!(
                    name = D::NAME,
                    ?reg,
                    len = data.len(),
                    val,
                    "virtio-mmio read"
                );

                data.copy_from_slice(&val.to_le_bytes());

                Ok(())
            } else {
                Err(VirtioError::MmioReadInvalidBufSize)
            }
        } else {
            let offset = (offset - CONFIGURATION_SPACE_OFFSET as u64)
                .try_into()
                .map_err(|_| VirtioError::MmioOffsetTooLarge)?;

            self.common.lock().unwrap().device.read_config(offset, data)
        }
    }

    pub fn write(&self, offset: u64, data: &[u8]) -> Result<()> {
        trace!(
            name = D::NAME,
            offset,
            len = data.len(),
            ?data,
            "virtio-mmio write"
        );

        if offset < CONFIGURATION_SPACE_OFFSET as u64 {
            let Some(reg) = MmioControlRegister::from_repr(offset as u16) else {
                return Err(VirtioError::MmioWriteInvalidRegisterOffset);
            };

            if data.len() == 4 {
                self.write_reg(reg, u32::from_le_bytes(data.try_into().unwrap()))
            } else {
                Err(VirtioError::MmioWriteInvalidBufSize)
            }
        } else {
            let offset = (offset - CONFIGURATION_SPACE_OFFSET as u64)
                .try_into()
                .map_err(|_| VirtioError::MmioOffsetTooLarge)?;

            self.common
                .lock()
                .unwrap()
                .device
                .write_config(offset, data)
        }
    }
}
