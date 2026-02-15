use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::device::capability::PciCapId;
use vm_pci::device::function::BarHandler;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::error::Error;
use vm_pci::types::configuration_space::ConfigurationSpace;
use zerocopy::IntoBytes;

use crate::device::pci::VirtIoPciDevice;
use crate::transport::VirtIoTransport;
use crate::transport::control_register::ControlRegister;
use crate::transport::pci::common_config_handler::CommonConfigHandler;
use crate::transport::pci::pci_header::VENDOR_ID;
use crate::types::interrupt_status::InterruptStatus;
use crate::types::pci::VirtIoPciCap;
use crate::types::pci::VirtIoPciCapCfgType;
use crate::types::pci::VirtIoPciCommonCfg;
use crate::types::pci::VirtIoPciNotifyCap;

pub mod pci_header;

mod common_config_handler;

struct NotifyHandler<D: VirtIoPciDevice> {
    transport: Arc<Mutex<VirtIoTransport<D>>>,
}

impl<D> BarHandler for NotifyHandler<D>
where
    D: VirtIoPciDevice,
{
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        unreachable!()
    }

    fn write(&self, _offset: u64, data: &[u8]) {
        assert_eq!(data.len(), 2);
        let queue_index = u16::from_le_bytes(data.try_into().unwrap());
        let mut transport = self.transport.lock().unwrap();
        transport
            .write_reg(ControlRegister::QueueNotify, queue_index.into())
            .unwrap();
    }
}

struct IsrHandler<D: VirtIoPciDevice> {
    transport: Arc<Mutex<VirtIoTransport<D>>>,
}

impl<D> BarHandler for IsrHandler<D>
where
    D: VirtIoPciDevice,
{
    fn read(&self, _offset: u64, data: &mut [u8]) {
        let mut transport = self.transport.lock().unwrap();
        let isr = transport.read_reg(ControlRegister::InterruptStatus);
        data[0] = isr as u8;
        transport
            .interrupt_status
            .remove(InterruptStatus::from_bits_truncate(isr));

        if transport.interrupt_status.is_empty() {
            transport.device.trigger_irq(false);
        }
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        unreachable!()
    }
}

struct DeviceHandler<D: VirtIoPciDevice> {
    transport: Arc<Mutex<VirtIoTransport<D>>>,
}

impl<D> BarHandler for DeviceHandler<D>
where
    D: VirtIoPciDevice,
{
    fn read(&self, offset: u64, data: &mut [u8]) {
        let transport = self.transport.lock().unwrap();

        transport
            .read_config(offset.try_into().unwrap(), data.len(), data)
            .unwrap();
    }

    fn write(&self, offset: u64, data: &[u8]) {
        let mut transport = self.transport.lock().unwrap();

        transport
            .write_config(offset.try_into().unwrap(), data.len(), data)
            .unwrap();
    }
}

pub struct VirtIoPciFunction<D: VirtIoPciDevice> {
    pub transport: Arc<Mutex<VirtIoTransport<D>>>,
}

impl<D> PciTypeFunctionCommon for VirtIoPciFunction<D>
where
    D: VirtIoPciDevice,
{
    const VENDOR_ID: u16 = VENDOR_ID;
    const DEVICE_ID: u16 = 0x1040 + D::DEVICE_ID as u16;
    const CLASS_CODE: u32 = D::CLASS_CODE;

    fn legacy_interrupt(&self) -> Option<(u8, u8)> {
        let transport = self.transport.lock().unwrap();
        transport.device.irq().map(|irq| {
            (
                irq.try_into()
                    .expect("irq is too large for pci legacy interrupt"),
                D::IRQ_PIN,
            )
        })
    }

    fn init_capability(&self, cfg: &mut ConfigurationSpace) -> Result<(), Error> {
        {
            let virtio_pci_common_cfg = VirtIoPciCap {
                cfg_type: VirtIoPciCapCfgType::VirtioPciCapCommonCfg as u8,
                bar: 0,
                id: 0,
                offset: 0,
                length: size_of::<VirtIoPciCommonCfg>().try_into().unwrap(),
                ..Default::default()
            };

            cfg.alloc_capability(PciCapId::Vndr, &virtio_pci_common_cfg.as_bytes()[2..])?;
        }

        {
            let virtio_pci_notify_cap = VirtIoPciNotifyCap {
                cap: VirtIoPciCap {
                    cap_len: size_of::<VirtIoPciNotifyCap>().try_into().unwrap(),
                    cfg_type: VirtIoPciCapCfgType::VirtioPciCapNotifyCfg as u8,
                    bar: 1,
                    id: 0,
                    offset: 0,
                    length: 0x1000,
                    ..Default::default()
                },
                notify_off_multiplier: 0,
            };

            cfg.alloc_capability(PciCapId::Vndr, &virtio_pci_notify_cap.as_bytes()[2..])?;
        }

        {
            let virtio_pci_isr_cap = VirtIoPciCap {
                cfg_type: VirtIoPciCapCfgType::VirtioPciCapIsrCfg as u8,
                bar: 2,
                id: 0,
                offset: 0,
                length: 0x1000,
                ..Default::default()
            };

            cfg.alloc_capability(PciCapId::Vndr, &virtio_pci_isr_cap.as_bytes()[2..])?;
        }

        {
            let virtio_pci_device_cfg_cap = VirtIoPciCap {
                cfg_type: VirtIoPciCapCfgType::VirtioPciCapDeviceCfg as u8,
                bar: 3,
                id: 0,
                offset: 0,
                length: 0x1000,
                ..Default::default()
            };
            assert!(D::DEVICE_SPECIFICATION_CONFIGURATION_LEN <= 0x1000);

            cfg.alloc_capability(PciCapId::Vndr, &virtio_pci_device_cfg_cap.as_bytes()[2..])?;
        }

        Ok(())
    }
}

impl<D> PciType0Function for VirtIoPciFunction<D>
where
    D: VirtIoPciDevice,
{
    const BAR_SIZE: [Option<u32>; 6] = [
        // virtio_pci_common_cfg
        Some(0x1000),
        // virtio_pci_notify_cap
        Some(0x1000),
        // virtio_pci_isr_cap
        Some(0x1000),
        // device_spec_cfg
        Some(0x1000),
        None,
        None,
    ];

    fn bar_handler(&self, bar: Bar) -> Option<Box<dyn BarHandler>> {
        match bar {
            Bar::Bar0 => Some(Box::new(CommonConfigHandler {
                transport: self.transport.clone(),
            })),
            Bar::Bar1 => Some(Box::new(NotifyHandler {
                transport: self.transport.clone(),
            })),
            Bar::Bar2 => Some(Box::new(IsrHandler {
                transport: self.transport.clone(),
            })),
            Bar::Bar3 => Some(Box::new(DeviceHandler {
                transport: self.transport.clone(),
            })),
            _ => None,
        }
    }
}
