use vm_virtio::transport::pci::pci_header::VENDOR_ID;
use vm_virtio::transport::pci::pci_header::VirtIoPciDeviceId;
use vm_virtio::types::pci::VirtIoPciCap;
use vm_virtio::types::pci::VirtIoPciCapCfgType;
use vm_virtio::types::pci::VirtIoPciCommonCfg;
use vm_virtio::types::pci::VirtIoPciNotifyCap;
use zerocopy::FromBytes;

use crate::pci::types::configuration_space::ConfigurationSpace;
use crate::pci::types::configuration_space::capability::PCI_CAP_ID_VNDR;
use crate::pci::types::function::PciTypeFunctionCommon;
use crate::pci::types::function::type0::PciType0Function;

pub struct DummyPci;

impl PciTypeFunctionCommon for DummyPci {
    const VENDOR_ID: u16 = VENDOR_ID;
    const DEVICE_ID: u16 = VirtIoPciDeviceId::Blk as u16;
    const PROG_IF: u8 = 0;
    const SUBCLASS: u8 = 0x80;
    const CLASS_CODE: u8 = 0x01;

    fn init_capability(cfg: &mut ConfigurationSpace) {
        let mut offset = 0;
        {
            // cap for virtio_pci_common_cfg
            let cap_len = size_of::<VirtIoPciCap>().try_into().unwrap();
            let len: u32 = size_of::<VirtIoPciCommonCfg>().try_into().unwrap();

            let cap = cfg.alloc_capability(PCI_CAP_ID_VNDR, cap_len);
            let cap = VirtIoPciCap::mut_from_bytes(cap).unwrap();
            cap.cap_len = cap_len;
            cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapCommonCfg as u8;
            cap.bar = 0;
            cap.id = 0;
            cap.offset = offset;
            cap.length = len;

            offset += len;
        }

        {
            // 4-byte aligned
            offset = offset.next_multiple_of(4);
            // cap for virtio_pci_notify_cap
            let cap_len = size_of::<VirtIoPciNotifyCap>().try_into().unwrap();
            let len = 32; // TODO: len

            let cap = cfg.alloc_capability(PCI_CAP_ID_VNDR, cap_len);
            let cap = VirtIoPciNotifyCap::mut_from_bytes(cap).unwrap();
            cap.cap.cap_len = cap_len;
            cap.cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapNotifyCfg as u8;
            cap.cap.bar = 0;
            cap.cap.id = 0;
            cap.cap.offset = offset;
            cap.cap.length = len;
            cap.notify_off_multiplier = 0;

            offset += len;
        }

        {
            // cap for virtio_pci_isr_cap
            let cap_len = size_of::<VirtIoPciCap>().try_into().unwrap();
            let len = 1;

            let cap = cfg.alloc_capability(PCI_CAP_ID_VNDR, cap_len);
            let cap = VirtIoPciCap::mut_from_bytes(cap).unwrap();
            cap.cap_len = cap_len;
            cap.cfg_type = VirtIoPciCapCfgType::VirtioPciCapIsrCfg as u8;
            cap.bar = 0;
            cap.id = 0;
            cap.offset = offset;
            cap.length = len;

            // offset += len;
        }
    }
}

impl PciType0Function for DummyPci {
    const BAR_SIZE: [Option<u32>; 6] = [Some(0x1000), None, None, None, None, None];
}
