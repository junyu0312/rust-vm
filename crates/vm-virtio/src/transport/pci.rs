use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::device::capability::Capability;
use vm_pci::device::capability::PciCapId;
use vm_pci::device::function::BarHandler;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;

use crate::device::pci::VirtIoPciDevice;
use crate::transport::VirtIoTransport;
use crate::transport::pci::common_config_handler::CommonConfigHandler;
use crate::transport::pci::pci_header::VENDOR_ID;
use crate::types::pci::VirtIoPciCap;
use crate::types::pci::VirtIoPciCapCfgType;
use crate::types::pci::VirtIoPciCommonCfg;
use crate::types::pci::VirtIoPciNotifyCap;

pub mod pci_header;

mod common_config_handler;

struct NotifyHandler;

impl BarHandler for NotifyHandler {
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        todo!()
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        todo!()
    }
}

struct IsrHandler;

impl BarHandler for IsrHandler {
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        todo!()
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        todo!()
    }
}

struct DeviceHandler<D: VirtIoPciDevice> {
    transport: Arc<Mutex<VirtIoTransport<D>>>,
}

impl<D> BarHandler for DeviceHandler<D>
where
    D: VirtIoPciDevice,
{
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        let _transport = self.transport.lock().unwrap();
        // todo!("{offset}")
        // data[0] = 1;
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        todo!()
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
    const IRQ_LINE: u8 = D::IRQ_LINE;
    const IRQ_PIN: u8 = D::IRQ_PIN;

    fn capabilities(&self) -> Vec<Capability> {
        let virtio_pci_common_cfg = VirtIoPciCap {
            cap_vndr: PciCapId::Vndr as u8,
            cap_len: size_of::<VirtIoPciCap>().try_into().unwrap(),
            cfg_type: VirtIoPciCapCfgType::VirtioPciCapCommonCfg as u8,
            bar: 0,
            id: 0,
            offset: 0,
            length: size_of::<VirtIoPciCommonCfg>().try_into().unwrap(),
            ..Default::default()
        };

        let virtio_pci_notify_cap = VirtIoPciNotifyCap {
            cap: VirtIoPciCap {
                cap_vndr: PciCapId::Vndr as u8,
                cap_len: size_of::<VirtIoPciCap>().try_into().unwrap(),
                cfg_type: VirtIoPciCapCfgType::VirtioPciCapNotifyCfg as u8,
                bar: 1,
                id: 0,
                offset: 0,
                length: 0x1000,
                ..Default::default()
            },
            notify_off_multiplier: 0,
        };

        let virtio_pci_isr_cap = VirtIoPciCap {
            cap_vndr: PciCapId::Vndr as u8,
            cap_len: size_of::<VirtIoPciCap>().try_into().unwrap(),
            cfg_type: VirtIoPciCapCfgType::VirtioPciCapIsrCfg as u8,
            bar: 2,
            id: 0,
            offset: 0,
            length: 0x1000,
            ..Default::default()
        };

        let virtio_pci_device_cfg_cap = VirtIoPciCap {
            cap_vndr: PciCapId::Vndr as u8,
            cap_len: size_of::<VirtIoPciCap>().try_into().unwrap(),
            cfg_type: VirtIoPciCapCfgType::VirtioPciCapDeviceCfg as u8,
            bar: 3,
            id: 0,
            offset: 0,
            length: 0x1000,
            ..Default::default()
        };
        assert!(D::DEVICE_SPECIFICATION_CONFIGURATION_LEN <= 0x1000);

        vec![
            Capability::from(virtio_pci_common_cfg),
            Capability::from(virtio_pci_notify_cap),
            Capability::from(virtio_pci_isr_cap),
            Capability::from(virtio_pci_device_cfg_cap),
        ]
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
            Bar::Bar1 => Some(Box::new(NotifyHandler)),
            Bar::Bar2 => Some(Box::new(IsrHandler)),
            Bar::Bar3 => Some(Box::new(DeviceHandler {
                transport: self.transport.clone(),
            })),
            _ => None,
        }
    }
}
