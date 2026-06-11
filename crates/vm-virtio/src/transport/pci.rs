use std::io::Read;
use std::io::Write;
use std::iter;
use std::sync::Arc;
use std::sync::Mutex;

use vm_core::device::Device;
use vm_core::device::error::DeviceSnapshotError;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::device::function::type0::Type0Function;
use vm_pci::error::Error;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::device::PciDevice;
use vm_pci::types::function::PciFunction;

use crate::device::VirtioDevice;
use crate::transport::VirtioDev;
use crate::types::pci::VirtioPciCap;
use crate::types::pci::VirtioPciCapCfgType;
use crate::types::pci::VirtioPciCommonCfg;
use crate::types::pci::VirtioPciNotifyCap;

mod common_config_handler;
mod device_handler;
mod isr_handler;
mod notify_handler;

const VIRTIO_PCI_VENDOR_ID: u16 = 0x1AF4;

pub struct VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    pub dev: Arc<Mutex<VirtioDev<D>>>,
}

impl<D> PciTypeFunctionCommon for VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    fn vendor_id(&self) -> u16 {
        VIRTIO_PCI_VENDOR_ID
    }

    fn device_id(&self) -> u16 {
        0x1040 + D::DEVICE_ID
    }

    fn class_code(&self) -> u32 {
        D::CLASS_CODE
    }

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
                length: size_of::<VirtioPciCommonCfg>()
                    .try_into()
                    .map_err(|_| Error::CapTooLarge)?,
                ..Default::default()
            };

            cfg.alloc_capability(virtio_pci_common_cfg.into())?;
        }

        {
            let virtio_pci_notify_cap = VirtioPciNotifyCap {
                cap: VirtioPciCap {
                    cap_len: size_of::<VirtioPciNotifyCap>()
                        .try_into()
                        .map_err(|_| Error::CapTooLarge)?,
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

        if D::DEVICE_SPECIFICATION_CONFIGURATION_LEN != 0 {
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

impl<D> PciType0Function for VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    fn bar_size(&self) -> [Option<u32>; 6] {
        [
            // virtio_pci_common_cfg
            Some(0x1000),
            // virtio_pci_notify_cap
            Some(0x1000),
            // virtio_pci_isr_cap
            Some(0x1000),
            // device_spec_cfg
            if D::DEVICE_SPECIFICATION_CONFIGURATION_LEN == 0 {
                None
            } else {
                Some(0x1000)
            },
            None,
            None,
        ]
    }

    fn bar_read(&self, bar: Bar, offset: u64, buf: &mut [u8]) {
        match bar {
            Bar::Bar0 => self.read_common_config(offset, buf),
            Bar::Bar1 => self.read_notify(offset, buf),
            Bar::Bar2 => self.read_isr(offset, buf),
            Bar::Bar3 => self.read_device(offset, buf),
            _ => unreachable!(),
        }
    }

    fn bar_write(&self, bar: Bar, offset: u64, buf: &[u8]) {
        match bar {
            Bar::Bar0 => self.write_common_config(offset, buf),
            Bar::Bar1 => self.write_notify(offset, buf),
            Bar::Bar2 => self.write_isr(offset, buf),
            Bar::Bar3 => self.write_device(offset, buf),
            _ => unreachable!(),
        }
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().pause()
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().resume()
    }

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().save(writer)
    }

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        self.dev.lock().unwrap().load(reader)
    }
}

pub struct VirtioPciDev<D>
where
    D: VirtioPciDevice,
{
    function: Type0Function<VirtioPciTransport<D>>,
}

impl<D> Device for VirtioPciDev<D>
where
    D: VirtioPciDevice,
{
    fn name(&self) -> String {
        "virtio pci dev".to_string()
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        self.function.pause()
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        self.function.resume()
    }

    fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        self.function.save(writer)
    }

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        self.function.load(reader)
    }
}

impl<D> PciDevice for VirtioPciDev<D>
where
    D: VirtioPciDevice,
{
    fn get_function(&self, function: u8) -> Option<&dyn PciFunction> {
        if function == 0 {
            return Some(&self.function);
        }

        None
    }

    fn get_function_mut(&mut self, function: u8) -> Option<&mut dyn PciFunction> {
        if function == 0 {
            return Some(&mut self.function);
        }

        None
    }

    fn functions(&self) -> Box<dyn Iterator<Item = &(dyn PciFunction + '_)> + '_> {
        Box::new(iter::once(&self.function as &dyn PciFunction))
    }
}

pub trait VirtioPciDevice: VirtioDevice {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize = 0;
    const CLASS_CODE: u32;
    const IRQ_PIN: u8;

    fn into_pci_device(self) -> VirtioPciDev<Self> {
        let virtio_function = VirtioPciTransport::<_> {
            dev: VirtioDev::new(self),
        };
        let function = Type0Function::new(virtio_function).unwrap();
        VirtioPciDev { function }
    }
}
