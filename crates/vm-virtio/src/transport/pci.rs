use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::capability::PciCapId;
use vm_pci::types::function::BarHandler;
use vm_pci::types::function::PciTypeFunctionCommon;
use vm_pci::types::function::type0::PciType0Function;
use zerocopy::FromBytes;

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
    fn read(&self, _offset: u64, _len: usize, _data: &mut [u8]) {
        todo!()
    }

    fn write(&self, _offset: u64, _len: usize, _data: &[u8]) {
        todo!()
    }
}

struct IsrHandler;

impl BarHandler for IsrHandler {
    fn read(&self, _offset: u64, _len: usize, _data: &mut [u8]) {
        todo!()
    }

    fn write(&self, _offset: u64, _len: usize, _data: &[u8]) {
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
    fn read(&self, _offset: u64, _len: usize, _data: &mut [u8]) {
        let _transport = self.transport.lock().unwrap();
        // todo!("{offset}")
        // data[0] = 1;
    }

    fn write(&self, _offset: u64, _len: usize, _data: &[u8]) {
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

    fn init_capability(cfg: &mut ConfigurationSpace) {
        {
            // cap for virtio_pci_common_cfg
            let cap_len = size_of::<VirtIoPciCap>().try_into().unwrap();

            let cap = cfg.alloc_capability(PciCapId::Vndr, cap_len);
            let cap = VirtIoPciCap::mut_from_bytes(cap).unwrap();
            cap.cap_len = cap_len;
            cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapCommonCfg as u8;
            cap.bar = 0;
            cap.id = 0;
            cap.offset = 0;
            cap.length = 0x1000;
            assert!(size_of::<VirtIoPciCommonCfg>() <= 0x1000);
        }

        {
            // cap for virtio_pci_notify_cap
            let cap_len = size_of::<VirtIoPciNotifyCap>().try_into().unwrap();

            let cap = cfg.alloc_capability(PciCapId::Vndr, cap_len);
            let cap = VirtIoPciNotifyCap::mut_from_bytes(cap).unwrap();
            cap.cap.cap_len = cap_len;
            cap.cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapNotifyCfg as u8;
            cap.cap.bar = 1;
            cap.cap.id = 0;
            cap.cap.offset = 0;
            cap.cap.length = 0x1000;
            cap.notify_off_multiplier = 0;
        }

        {
            // cap for virtio_pci_isr_cap
            let cap_len = size_of::<VirtIoPciCap>().try_into().unwrap();

            let cap = cfg.alloc_capability(PciCapId::Vndr, cap_len);
            let cap = VirtIoPciCap::mut_from_bytes(cap).unwrap();
            cap.cap_len = cap_len;
            cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapIsrCfg as u8;
            cap.bar = 2;
            cap.id = 0;
            cap.offset = 0;
            cap.length = 0x1000;
        }

        {
            // cap for device_spec_cfg
            let cap_len = size_of::<VirtIoPciCap>().try_into().unwrap();

            let cap = cfg.alloc_capability(PciCapId::Vndr, cap_len);
            let cap = VirtIoPciCap::mut_from_bytes(cap).unwrap();
            cap.cap_len = cap_len;
            cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapDeviceCfg as u8;
            cap.bar = 3;
            cap.id = 0;
            cap.offset = 0;
            cap.length = 0x1000;
            assert!(D::DEVICE_SPECIFICATION_CONFIGURATION_LEN <= 0x1000);
        }
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

    fn bar_handler(&self, n: u8) -> Option<Box<dyn BarHandler>> {
        match n {
            0 => Some(Box::new(CommonConfigHandler {
                transport: self.transport.clone(),
            })),
            1 => Some(Box::new(NotifyHandler)),
            2 => Some(Box::new(IsrHandler)),
            3 => Some(Box::new(DeviceHandler {
                transport: self.transport.clone(),
            })),
            _ => None,
        }
    }
}
