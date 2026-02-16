use strum_macros::FromRepr;
use tracing::warn;
use vm_pci::device::function::BarHandler;

use crate::device::pci::VirtIoPciDevice;
use crate::transport::VirtIoDev;
use crate::transport::control_register::ControlRegister;

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

pub struct CommonConfigHandler<D: VirtIoPciDevice> {
    pub transport: VirtIoDev<D>,
}

impl<D> BarHandler for CommonConfigHandler<D>
where
    D: VirtIoPciDevice,
{
    fn read(&self, offset: u64, data: &mut [u8]) {
        let Some(offset) = CommonCfgOffset::from_repr(offset) else {
            warn!(offset, "invalid offset");
            return;
        };

        let transport = self.transport.blocking_lock();

        match offset {
            CommonCfgOffset::DeviceFeatureSelect => todo!(),
            CommonCfgOffset::DeviceFeature => {
                assert_eq!(data.len(), 4);
                let feat = transport.read_reg(ControlRegister::DeviceFeatures);
                data.copy_from_slice(&feat.to_le_bytes());
            }
            CommonCfgOffset::DriverFeatureSelect => todo!(),
            CommonCfgOffset::DriverFeature => todo!(),
            CommonCfgOffset::ConfigMsixVector => todo!(),
            CommonCfgOffset::NumQueues => {
                assert_eq!(data.len(), 2);
                let num_queues: u16 = D::VIRT_QUEUES_SIZE_MAX.len().try_into().unwrap();
                data.copy_from_slice(&num_queues.to_le_bytes());
            }
            CommonCfgOffset::DeviceStatus => {
                assert_eq!(data.len(), 1);
                let status = transport.read_reg(ControlRegister::Status);
                data[0] = status.try_into().unwrap();
            }
            CommonCfgOffset::ConfigGeneration => {
                let cfg_generation: u8 = transport
                    .read_reg(ControlRegister::ConfigGeneration)
                    .try_into()
                    .unwrap();
                data[0] = cfg_generation;
            }
            CommonCfgOffset::QueueSelect => todo!(),
            CommonCfgOffset::QueueSize => {
                assert_eq!(data.len(), 2);
                let queue_size: u16 = transport
                    .read_reg(ControlRegister::QueueSize)
                    .try_into()
                    .unwrap();
                data.copy_from_slice(&queue_size.to_le_bytes());
            }
            CommonCfgOffset::QueueMsixVector => todo!(),
            CommonCfgOffset::QueueEnable => {
                assert_eq!(data.len(), 2);
                let queue_ready = transport.read_reg(ControlRegister::QueueReady) as u16;
                data.copy_from_slice(&queue_ready.to_le_bytes());
            }
            CommonCfgOffset::QueueNotifyOff => {
                // let val: u16 = transport
                //     .read_reg(ControlRegister::QueueNotify)
                //     .try_into()
                //     .unwrap();
                // data.copy_from_slice(&val.to_le_bytes());
                // TODO: What's this
            }
            CommonCfgOffset::QueueDescLow => todo!(),
            CommonCfgOffset::QueueDescHigh => todo!(),
            CommonCfgOffset::QueueDriverLow => todo!(),
            CommonCfgOffset::QueueDriverHigh => todo!(),
            CommonCfgOffset::QueueDeviceLow => todo!(),
            CommonCfgOffset::QueueDeviceHigh => todo!(),
        }
    }

    fn write(&self, offset: u64, data: &[u8]) {
        let Some(offset) = CommonCfgOffset::from_repr(offset) else {
            warn!(offset, "invalid offset");
            return;
        };

        let mut transport = self.transport.blocking_lock();

        match offset {
            CommonCfgOffset::DeviceFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::DeviceFeaturesSel, sel)
                    .unwrap();
            }
            CommonCfgOffset::DriverFeatureSelect => {
                assert_eq!(data.len(), 4);
                let sel = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::DriverFeaturesSel, sel)
                    .unwrap();
            }
            CommonCfgOffset::DriverFeature => {
                assert_eq!(data.len(), 4);
                let sel = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::DriverFeatures, sel)
                    .unwrap();
            }
            CommonCfgOffset::ConfigMsixVector => todo!(),
            CommonCfgOffset::DeviceStatus => {
                assert_eq!(data.len(), 1);
                let status = data[0];
                transport
                    .write_reg(ControlRegister::Status, status as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueSelect => {
                assert_eq!(data.len(), 2);
                let sel = u16::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueSel, sel as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueSize => {
                assert_eq!(data.len(), 2);
                let queue_size = u16::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueSize, queue_size as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueMsixVector => todo!(),
            CommonCfgOffset::QueueEnable => {
                let queue_enable = u16::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueReady, queue_enable as u32)
                    .unwrap();
            }
            CommonCfgOffset::QueueDescLow => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueDescLow, addr)
                    .unwrap();
            }
            CommonCfgOffset::QueueDescHigh => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueDescHigh, addr)
                    .unwrap();
            }
            CommonCfgOffset::QueueDriverLow => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueAvailLow, addr)
                    .unwrap();
            }
            CommonCfgOffset::QueueDriverHigh => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueAvailHigh, addr)
                    .unwrap();
            }
            CommonCfgOffset::QueueDeviceLow => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueUsedLow, addr)
                    .unwrap();
            }
            CommonCfgOffset::QueueDeviceHigh => {
                let addr = u32::from_le_bytes(data.try_into().unwrap());
                transport
                    .write_reg(ControlRegister::QueueUsedHigh, addr)
                    .unwrap();
            }
            _ => {
                warn!(?offset, "write to a RO cfg");
            }
        }
    }
}
