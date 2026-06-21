use std::sync::Arc;

use strum_macros::FromRepr;
use tracing::warn;

use crate::device::virtqueue::VirtqueueWorkerController;
use crate::device::virtqueue::virtqueue_worker;
use crate::transport::common::VirtqueueHandler;
use crate::transport::common::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;
use crate::transport::pci::VirtioPciTransport;

#[derive(Debug, FromRepr)]
#[repr(u64)]
enum CommonCfgOffset {
    // About the whole device
    DeviceFeatureSelect = 0x00,
    DeviceFeature = 0x04,
    DriverFeatureSelect = 0x08,
    DriverFeature = 0x0c,
    ConfigMsixVector = 0x10,
    NumQueues = 0x12,
    DeviceStatus = 0x14,
    ConfigGeneration = 0x15,

    // About a specific virtqueue
    QueueSelect = 0x16,
    QueueSize = 0x18,
    QueueMsixVector = 0x1a,
    QueueEnable = 0x1c,
    QueueNotifyOff = 0x1e,
    QueueDescLow = 0x20,
    QueueDescHigh = 0x24,
    QueueDriverLow = 0x28,
    QueueDriverHigh = 0x2c,
    QueueDeviceLow = 0x30,
    QueueDeviceHigh = 0x34,
    // QueueNotifConfigData = 0x38,
    // QueueReset = 0x3a,
    // AdminQueueIndex = 0x3c,
    // AdminQueueNum = 0x3e,
}

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    pub fn read_common_config(&self, offset: u64, data: &mut [u8]) {
        let Some(offset) = CommonCfgOffset::from_repr(offset) else {
            warn!(name = D::NAME, offset, "invalid offset");
            return;
        };

        let dev = self.common.lock().unwrap();

        match offset {
            CommonCfgOffset::DeviceFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = dev.read_reg(ControlRegister::DeviceFeaturesSel).unwrap();
                data.copy_from_slice(&sel.to_le_bytes());
            }
            CommonCfgOffset::DeviceFeature => {
                assert_eq!(data.len(), 4);
                let feat = dev.read_reg(ControlRegister::DeviceFeatures).unwrap();
                data.copy_from_slice(&feat.to_le_bytes());
            }
            CommonCfgOffset::DriverFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = dev.read_reg(ControlRegister::DriverFeaturesSel).unwrap();
                data.copy_from_slice(&sel.to_le_bytes());
            }
            CommonCfgOffset::DriverFeature => {
                assert_eq!(data.len(), 4);
                let feat = dev.read_reg(ControlRegister::DriverFeatures).unwrap();
                data.copy_from_slice(&feat.to_le_bytes());
            }
            CommonCfgOffset::ConfigMsixVector => {
                assert_eq!(data.len(), 2);
                let virtio_pci_msix_vector = self
                    .interrupt_dispatcher
                    .virtio_pci_msix_vector
                    .read()
                    .unwrap();
                data.copy_from_slice(&virtio_pci_msix_vector.config_msix_vector.to_le_bytes());
            }
            CommonCfgOffset::NumQueues => {
                assert_eq!(data.len(), 2);
                let num_queues = dev.device.num_queues();
                data.copy_from_slice(&num_queues.to_le_bytes());
            }
            CommonCfgOffset::DeviceStatus => {
                assert_eq!(data.len(), 1);
                let status = dev.read_reg(ControlRegister::Status).unwrap();
                data[0] = status.try_into().unwrap();
            }
            CommonCfgOffset::ConfigGeneration => {
                let cfg_generation: u8 = dev
                    .read_reg(ControlRegister::ConfigGeneration)
                    .unwrap()
                    .try_into()
                    .unwrap();
                data[0] = cfg_generation;
            }
            CommonCfgOffset::QueueSelect => {
                assert_eq!(data.len(), 2);
                let queue_sel: u16 = dev
                    .read_reg(ControlRegister::QueueSel)
                    .unwrap()
                    .try_into()
                    .unwrap();
                data.copy_from_slice(&queue_sel.to_le_bytes());
            }
            CommonCfgOffset::QueueSize => {
                assert_eq!(data.len(), 2);
                let queue_size: u16 = dev
                    .read_reg(ControlRegister::QueueSize)
                    .unwrap()
                    .try_into()
                    .unwrap();
                data.copy_from_slice(&queue_size.to_le_bytes());
            }
            CommonCfgOffset::QueueMsixVector => {
                assert_eq!(data.len(), 2);
                let sel = dev.get_queue_sel();
                let virtio_pci_msix_vector = self
                    .interrupt_dispatcher
                    .virtio_pci_msix_vector
                    .read()
                    .unwrap();
                data.copy_from_slice(
                    &virtio_pci_msix_vector.queue_msix_vector[sel as usize].to_le_bytes(),
                );
            }
            CommonCfgOffset::QueueEnable => {
                assert_eq!(data.len(), 2);
                let queue_ready = dev.read_reg(ControlRegister::QueueReady).unwrap() as u16;
                data.copy_from_slice(&queue_ready.to_le_bytes());
            }
            CommonCfgOffset::QueueNotifyOff => {
                // TODO
            }
            CommonCfgOffset::QueueDescLow => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueDescLow).unwrap();
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDescHigh => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueDescHigh).unwrap();
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDriverLow => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueAvailLow).unwrap();
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDriverHigh => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueAvailHigh).unwrap();
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDeviceLow => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueUsedLow).unwrap();
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDeviceHigh => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueUsedHigh).unwrap();
                data.copy_from_slice(&addr.to_le_bytes());
            }
        }
    }

    pub fn write_common_config(&self, offset: u64, data: &[u8]) {
        let Some(offset) = CommonCfgOffset::from_repr(offset) else {
            warn!(name = D::NAME, offset, "invalid offset");
            return;
        };

        let mut dev = self.common.lock().unwrap();

        match offset {
            CommonCfgOffset::DeviceFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::DeviceFeaturesSel, sel)
                    .unwrap();
            }
            CommonCfgOffset::DriverFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::DriverFeaturesSel, sel)
                    .unwrap();
            }
            CommonCfgOffset::DriverFeature => {
                assert_eq!(data.len(), 4);
                let sel = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::DriverFeatures, sel).unwrap();
            }
            CommonCfgOffset::ConfigMsixVector => {
                assert_eq!(data.len(), 2);
                let config_msix_vector = u16::from_le_bytes(data.try_into().unwrap());
                let mut virtio_pci_msix_vector = self
                    .interrupt_dispatcher
                    .virtio_pci_msix_vector
                    .write()
                    .unwrap();
                virtio_pci_msix_vector.config_msix_vector = config_msix_vector;
            }
            CommonCfgOffset::DeviceStatus => {
                assert_eq!(data.len(), 1);
                let status = data[0];
                dev.write_reg(ControlRegister::Status, status as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueSelect => {
                assert_eq!(data.len(), 2);
                let sel = u16::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueSel, sel as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueSize => {
                assert_eq!(data.len(), 2);
                let queue_size = u16::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueSize, queue_size as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueMsixVector => {
                assert_eq!(data.len(), 2);
                let queue_msix_vector = u16::from_le_bytes(data.try_into().unwrap());
                let sel = dev.get_queue_sel();

                let mut virtio_pci_msix_vector = self
                    .interrupt_dispatcher
                    .virtio_pci_msix_vector
                    .write()
                    .unwrap();
                virtio_pci_msix_vector.queue_msix_vector[sel as usize] = queue_msix_vector;
            }
            CommonCfgOffset::QueueEnable => {
                let queue_enable = u16::from_le_bytes(data.try_into().unwrap());
                let mut virtqueue = self.virtqueue_handlers.write().unwrap();
                let queue_sel = dev.get_queue_sel();

                if queue_enable == 0 {
                    // disable
                    todo!()
                } else {
                    let Some(handler) = dev.device.virtqueue_handler(queue_sel) else {
                        unreachable!("no handler for queue {queue_sel}");
                    };

                    let controller = Arc::new(VirtqueueWorkerController::default());

                    let _join_handler = self.tokio_runtime.spawn(virtqueue_worker(
                        self.memory.clone(),
                        controller.clone(),
                        self.get_used_buffer_notification(dev.get_interrupt_status(), queue_sel),
                        *dev.get_virtqueue(queue_sel).unwrap(),
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

                dev.write_reg(ControlRegister::QueueReady, queue_enable as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueDescLow => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueDescLow, addr).unwrap();
            }
            CommonCfgOffset::QueueDescHigh => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueDescHigh, addr).unwrap();
            }
            CommonCfgOffset::QueueDriverLow => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueAvailLow, addr).unwrap();
            }
            CommonCfgOffset::QueueDriverHigh => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueAvailHigh, addr)
                    .unwrap();
            }
            CommonCfgOffset::QueueDeviceLow => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueUsedLow, addr).unwrap();
            }
            CommonCfgOffset::QueueDeviceHigh => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                dev.write_reg(ControlRegister::QueueUsedHigh, addr).unwrap();
            }
            _ => warn!(name = D::NAME, ?offset, "write to a RO cfg"),
        }
    }
}
