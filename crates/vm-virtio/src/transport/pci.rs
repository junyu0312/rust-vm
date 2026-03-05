use std::sync::Arc;
use std::sync::Mutex;

use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::BarHandler;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::error::Error;
use vm_pci::types::configuration_space::ConfigurationSpace;

use crate::device::pci::VirtioPciDevice;
use crate::transport::VirtioDev;
use crate::transport::pci::common_config_handler::CommonConfigHandler;
use crate::transport::pci::device_handler::DeviceHandler;
use crate::transport::pci::isr_handler::IsrHandler;
use crate::transport::pci::notify_handler::NotifyHandler;
use crate::types::pci::VirtioPciCap;
use crate::types::pci::VirtioPciCapCfgType;
use crate::types::pci::VirtioPciCommonCfg;
use crate::types::pci::VirtioPciNotifyCap;

mod common_config_handler;
mod device_handler;
mod isr_handler;
mod notify_handler;

const VIRTIO_PCI_VENDOR_ID: u16 = 0x1AF4;

pub struct VirtioPciTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    pub dev: Arc<Mutex<VirtioDev<C, D>>>,
}

impl<C, D> PciTypeFunctionCommon for VirtioPciTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    const VENDOR_ID: u16 = VIRTIO_PCI_VENDOR_ID;
    const DEVICE_ID: u16 = 0x1040 + D::DEVICE_ID as u16;
    const CLASS_CODE: u32 = D::CLASS_CODE;

    fn legacy_interrupt(&self) -> Option<(u8, u8)> {
        let dev = self.dev.lock().unwrap();
        dev.device.irq().map(|irq| {
            (
                irq.try_into()
                    .expect("irq is too large for pci legacy interrupt"),
                D::IRQ_PIN,
            )
        })
    }

    fn init_capability(&self, cfg: &mut ConfigurationSpace) -> Result<(), Error> {
        {
            let virtio_pci_common_cfg = VirtioPciCap {
                cfg_type: VirtioPciCapCfgType::VirtioPciCapCommonCfg as u8,
                bar: 0,
                id: 0,
                offset: 0,
                length: size_of::<VirtioPciCommonCfg>().try_into().unwrap(),
                ..Default::default()
            };

            cfg.alloc_capability(virtio_pci_common_cfg.into())?;
        }

        {
            let virtio_pci_notify_cap = VirtioPciNotifyCap {
                cap: VirtioPciCap {
                    cap_len: size_of::<VirtioPciNotifyCap>().try_into().unwrap(),
                    cfg_type: VirtioPciCapCfgType::VirtioPciCapNotifyCfg as u8,
                    bar: 1,
                    id: 0,
                    offset: 0,
                    length: 0x1000,
                    ..Default::default()
                },
                notify_off_multiplier: 0,
            };

            cfg.alloc_capability(virtio_pci_notify_cap.into())?;
        }

        {
            let virtio_pci_isr_cap = VirtioPciCap {
                cfg_type: VirtioPciCapCfgType::VirtioPciCapIsrCfg as u8,
                bar: 2,
                id: 0,
                offset: 0,
                length: 0x1000,
                ..Default::default()
            };

            cfg.alloc_capability(virtio_pci_isr_cap.into())?;
        }

        {
            let virtio_pci_device_cfg_cap = VirtioPciCap {
                cfg_type: VirtioPciCapCfgType::VirtioPciCapDeviceCfg as u8,
                bar: 3,
                id: 0,
                offset: 0,
                length: 0x1000,
                ..Default::default()
            };

            if D::DEVICE_SPECIFICATION_CONFIGURATION_LEN > 0x1000 {
                return Err(Error::CapTooLarge);
            }

            cfg.alloc_capability(virtio_pci_device_cfg_cap.into())?;
        }

        Ok(())
    }
}

impl<C, D> PciType0Function for VirtioPciTransport<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
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
                dev: self.dev.clone(),
            })),
            Bar::Bar1 => Some(Box::new(NotifyHandler {
                dev: self.dev.clone(),
            })),
            Bar::Bar2 => Some(Box::new(IsrHandler {
                dev: self.dev.clone(),
            })),
            Bar::Bar3 => Some(Box::new(DeviceHandler {
                dev: self.dev.clone(),
            })),
            _ => None,
        }
    }
}
