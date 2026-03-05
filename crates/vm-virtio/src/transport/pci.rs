use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::BarHandler;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::error::Error;
use vm_pci::types::configuration_space::ConfigurationSpace;

use crate::device::pci::VirtioPciDevice;
use crate::transport::VirtioDev;
use crate::transport::control_register::ControlRegister;
use crate::transport::pci::common_config_handler::CommonConfigHandler;
use crate::transport::pci::pci_header::VENDOR_ID;
use crate::types::pci::VirtioPciCap;
use crate::types::pci::VirtioPciCapCfgType;
use crate::types::pci::VirtioPciCommonCfg;
use crate::types::pci::VirtioPciNotifyCap;

pub mod pci_header;

mod common_config_handler;

struct NotifyHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    dev: VirtioDev<C, D>,
}

impl<C, D> BarHandler for NotifyHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        unreachable!()
    }

    fn write(&self, _offset: u64, data: &[u8]) {
        assert_eq!(data.len(), 2);
        let queue_index = u16::from_le_bytes(data.try_into().unwrap());
        let mut transport = self.dev.lock().unwrap();
        transport
            .write_reg(ControlRegister::QueueNotify, queue_index.into())
            .unwrap();
    }
}

struct IsrHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    transport: VirtioDev<C, D>,
}

impl<C, D> BarHandler for IsrHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    fn read(&self, _offset: u64, data: &mut [u8]) {
        let mut transport = self.transport.lock().unwrap();

        let isr = transport.read_reg(ControlRegister::InterruptStatus);
        data[0] = isr as u8;

        /*
         * From `4.1.4.5.1 Device Requirements: ISR status capability`
         * - The device MUST reset ISR status to 0 on driver read.
         */
        transport
            .write_reg(ControlRegister::InterruptStatus, 0)
            .unwrap();
        transport.device.trigger_irq(false);
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        unreachable!()
    }
}

struct DeviceHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    transport: VirtioDev<C, D>,
}

impl<C, D> BarHandler for DeviceHandler<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
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

pub struct VirtioPciFunction<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    pub dev: VirtioDev<C, D>,
}

impl<C, D> PciTypeFunctionCommon for VirtioPciFunction<C, D>
where
    C: MemoryContainer,
    D: VirtioPciDevice<C>,
{
    const VENDOR_ID: u16 = VENDOR_ID;
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
            assert!(D::DEVICE_SPECIFICATION_CONFIGURATION_LEN <= 0x1000);

            cfg.alloc_capability(virtio_pci_device_cfg_cap.into())?;
        }

        Ok(())
    }
}

impl<C, D> PciType0Function for VirtioPciFunction<C, D>
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
                transport: self.dev.clone(),
            })),
            Bar::Bar1 => Some(Box::new(NotifyHandler {
                dev: self.dev.clone(),
            })),
            Bar::Bar2 => Some(Box::new(IsrHandler {
                transport: self.dev.clone(),
            })),
            Bar::Bar3 => Some(Box::new(DeviceHandler {
                transport: self.dev.clone(),
            })),
            _ => None,
        }
    }
}
