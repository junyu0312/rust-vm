use std::sync::Arc;
use std::sync::Mutex;

use strum_macros::FromRepr;
use tracing::warn;
use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::BarHandler;

use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::pci::VirtioPciDevice;

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

pub struct CommonConfigHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    pub dev: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> BarHandler for CommonConfigHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    fn read(&self, offset: u64, data: &mut [u8]) {
        let Some(offset) = CommonCfgOffset::from_repr(offset) else {
            warn!(name = D::NAME, offset, "invalid offset");
            return;
        };

        let dev = self.dev.lock().unwrap();

        match offset {
            CommonCfgOffset::DeviceFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = dev.read_reg(ControlRegister::DeviceFeaturesSel);
                data.copy_from_slice(&sel.to_le_bytes());
            }
            CommonCfgOffset::DeviceFeature => {
                assert_eq!(data.len(), 4);
                let feat = dev.read_reg(ControlRegister::DeviceFeatures);
                data.copy_from_slice(&feat.to_le_bytes());
            }
            CommonCfgOffset::DriverFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = dev.read_reg(ControlRegister::DriverFeaturesSel);
                data.copy_from_slice(&sel.to_le_bytes());
            }
            CommonCfgOffset::DriverFeature => {
                assert_eq!(data.len(), 4);
                let feat = dev.read_reg(ControlRegister::DriverFeatures);
                data.copy_from_slice(&feat.to_le_bytes());
            }
            CommonCfgOffset::ConfigMsixVector => todo!(),
            CommonCfgOffset::NumQueues => {
                assert_eq!(data.len(), 2);
                let num_queues = dev.device.num_queues();
                data.copy_from_slice(&num_queues.to_le_bytes());
            }
            CommonCfgOffset::DeviceStatus => {
                assert_eq!(data.len(), 1);
                let status = dev.read_reg(ControlRegister::Status);
                data[0] = status.try_into().unwrap();
            }
            CommonCfgOffset::ConfigGeneration => {
                let cfg_generation: u8 = dev
                    .read_reg(ControlRegister::ConfigGeneration)
                    .try_into()
                    .unwrap();
                data[0] = cfg_generation;
            }
            CommonCfgOffset::QueueSelect => {
                assert_eq!(data.len(), 2);
                let queue_sel: u16 = dev.read_reg(ControlRegister::QueueSel).try_into().unwrap();
                data.copy_from_slice(&queue_sel.to_le_bytes());
            }
            CommonCfgOffset::QueueSize => {
                assert_eq!(data.len(), 2);
                let queue_size: u16 = dev.read_reg(ControlRegister::QueueSize).try_into().unwrap();
                data.copy_from_slice(&queue_size.to_le_bytes());
            }
            CommonCfgOffset::QueueMsixVector => todo!(),
            CommonCfgOffset::QueueEnable => {
                assert_eq!(data.len(), 2);
                let queue_ready = dev.read_reg(ControlRegister::QueueReady) as u16;
                data.copy_from_slice(&queue_ready.to_le_bytes());
            }
            CommonCfgOffset::QueueNotifyOff => {
                // TODO
            }
            CommonCfgOffset::QueueDescLow => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueDescLow);
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDescHigh => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueDescHigh);
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDriverLow => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueAvailLow);
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDriverHigh => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueAvailHigh);
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDeviceLow => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueUsedLow);
                data.copy_from_slice(&addr.to_le_bytes());
            }
            CommonCfgOffset::QueueDeviceHigh => {
                assert_eq!(data.len(), 4);
                let addr = dev.read_reg(ControlRegister::QueueUsedHigh);
                data.copy_from_slice(&addr.to_le_bytes());
            }
        }
    }

    fn write(&self, offset: u64, data: &[u8]) {
        let Some(offset) = CommonCfgOffset::from_repr(offset) else {
            warn!(name = D::NAME, offset, "invalid offset");
            return;
        };

        let mut dev = self.dev.lock().unwrap();

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
            CommonCfgOffset::ConfigMsixVector => todo!(),
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
            CommonCfgOffset::QueueMsixVector => todo!(),
            CommonCfgOffset::QueueEnable => {
                let queue_enable = u16::from_le_bytes(data.try_into().unwrap());
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
