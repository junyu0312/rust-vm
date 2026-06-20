use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::iter;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use tokio::runtime::Handle;
use vm_core::arch::irq::InterruptController;
use vm_core::device::Device;
use vm_core::device::error::DeviceSnapshotError;
use vm_core::virtualization::irq_allocator::IrqAllocator;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::device::capability::msix::PciMsixCap;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::device::function::type0::Type0Function;
use vm_pci::error::Error;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::device::PciDevice;
use vm_pci::types::function::PciFunction;

use crate::device::VirtioDevice;
use crate::device::virtqueue::VirtioConfigurationChangeNotifier;
use crate::device::virtqueue::VirtioUsedBufferNotifier;
use crate::result::Result;
use crate::result::VirtioError;
use crate::transport::VirtioDeviceOps;
use crate::transport::common::VirtioTransportCommon;
use crate::transport::common::VirtqueueHandler;
use crate::transport::pci::interrupt::VirtioPciConfigurationChangeNotifier;
use crate::transport::pci::interrupt::VirtioPciEventUsedBufferNotifier;
use crate::transport::pci::interrupt::VirtioPciIrqDispatcher;
use crate::transport::pci::msix::VirtioPciMsixInfo;
use crate::types::interrupt_status::InterruptStatus;
use crate::types::pci::VIRTIO_MSI_NO_VECTOR;
use crate::types::pci::VirtioPciCap;
use crate::types::pci::VirtioPciCapCfgType;
use crate::types::pci::VirtioPciCommonCfg;
use crate::types::pci::VirtioPciNotifyCap;

mod common_config_handler;
mod device_handler;
mod interrupt;
mod isr_handler;
mod msix;
mod msix_handler;
mod notify_handler;

const VIRTIO_PCI_VENDOR_ID: u16 = 0x1AF4;

struct VirtioPciMsixVector {
    // extend for virtio_pci_common_cfg
    config_msix_vector: u16,
    queue_msix_vector: Vec<u16>,
}

pub struct VirtioPciTransport<D> {
    configuration_space: Arc<Mutex<ConfigurationSpace>>,
    common: Mutex<VirtioTransportCommon<D>>,
    interrupt_dispatcher: Arc<VirtioPciIrqDispatcher>,

    tokio_runtime: Handle,
    memory: Arc<MemoryAddressSpace>,

    virtqueue_handlers: RwLock<HashMap<u16, VirtqueueHandler>>,
    configuration_change_notification: Arc<VirtioPciConfigurationChangeNotifier>,
}

impl<D> VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    fn new(
        irq_allocator: &mut IrqAllocator,
        tokio_runtime: Handle,
        memory: Arc<MemoryAddressSpace>,
        irq_chip: Arc<dyn InterruptController>,
        common: VirtioTransportCommon<D>,
    ) -> Self {
        let configuration_space = Arc::new(Mutex::new(ConfigurationSpace::default()));

        let num_queues = common.device.num_queues();

        let virtio_pci_msix_vector = RwLock::new(VirtioPciMsixVector {
            config_msix_vector: VIRTIO_MSI_NO_VECTOR,
            queue_msix_vector: vec![VIRTIO_MSI_NO_VECTOR; num_queues as usize],
        });

        let interrupt_dispatcher = {
            let legacy_int;
            let msix;

            if cfg!(target_os = "linux") {
                legacy_int = None;
                msix = Some(Arc::new(RwLock::new(VirtioPciMsixInfo::new(num_queues))));
            } else {
                legacy_int = Some(irq_allocator.alloc().unwrap().try_into().unwrap());
                msix = None
            };

            Arc::new(VirtioPciIrqDispatcher {
                irq_chip,
                configuration_space: configuration_space.clone(),
                virtio_pci_msix_vector,
                legacy_int,
                msix,
                msix_cap_offset: Default::default(),
            })
        };

        let configuration_change_notification = Arc::new(VirtioPciConfigurationChangeNotifier {
            interrupt_dispatcher: interrupt_dispatcher.clone(),
            is: common.get_interrupt_status(),
            config_generation: common.get_config_generation(),
        });

        VirtioPciTransport {
            configuration_space,
            common: Mutex::new(common),
            interrupt_dispatcher,
            tokio_runtime,
            memory,
            virtqueue_handlers: Default::default(),
            configuration_change_notification,
        }
    }

    fn get_used_buffer_notification(
        &self,
        is: Arc<Mutex<InterruptStatus>>,
        queue_sel: u16,
    ) -> Arc<dyn VirtioUsedBufferNotifier> {
        let notifier = VirtioPciEventUsedBufferNotifier {
            interrupt_dispatcher: self.interrupt_dispatcher.clone(),
            queue_sel,
            is,
        };

        Arc::new(notifier)
    }
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
        self.interrupt_dispatcher
            .legacy_int
            .map(|irq| (irq, D::IRQ_PIN))
    }

    fn init_capability(&self, cfg: &mut ConfigurationSpace) -> std::result::Result<(), Error> {
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

        if let Some(msix) = self.interrupt_dispatcher.msix.as_ref() {
            let msix = msix.read().unwrap();
            let msix_cap = PciMsixCap::new(msix.vectors(), 4, 0, 4, msix.pba_offset());
            let cap_offset = cfg.alloc_capability(msix_cap.into())?;
            *self.interrupt_dispatcher.msix_cap_offset.lock().unwrap() = Some(cap_offset);
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
            self.interrupt_dispatcher
                .msix
                .as_ref()
                .map(|msix| msix.read().unwrap().bar_size()),
            None,
        ]
    }

    fn bar_read(&self, bar: Bar, offset: u64, buf: &mut [u8]) {
        match bar {
            Bar::Bar0 => self.read_common_config(offset, buf),
            Bar::Bar1 => self.read_notify(offset, buf),
            Bar::Bar2 => self.read_isr(offset, buf),
            Bar::Bar3 => self.read_device(offset, buf),
            Bar::Bar4 => self.read_msix(offset, buf),
            _ => unreachable!(),
        }
    }

    fn bar_write(&self, bar: Bar, offset: u64, buf: &[u8]) {
        match bar {
            Bar::Bar0 => self.write_common_config(offset, buf),
            Bar::Bar1 => self.write_notify(offset, buf),
            Bar::Bar2 => self.write_isr(offset, buf),
            Bar::Bar3 => self.write_device(offset, buf),
            Bar::Bar4 => self.write_msix(offset, buf),
            _ => unreachable!(),
        }
    }

    fn pause(&self) -> std::result::Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn resume(&self) -> std::result::Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn save(&self, _writer: &mut dyn Write) -> std::result::Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn load(&mut self, _reader: &mut dyn Read) -> std::result::Result<(), DeviceSnapshotError> {
        todo!()
    }
}

impl<D> VirtioDeviceOps for VirtioPciTransport<D>
where
    D: VirtioPciDevice,
{
    fn configuration_change_notifier(&self) -> Arc<dyn VirtioConfigurationChangeNotifier> {
        self.configuration_change_notification.clone()
    }
}

pub struct VirtioPciDev<D>
where
    D: VirtioPciDevice,
{
    function: Type0Function<VirtioPciTransport<D>>,
}

impl<D> TryFrom<VirtioPciTransport<D>> for VirtioPciDev<D>
where
    D: VirtioPciDevice,
{
    type Error = VirtioError;

    fn try_from(dev: VirtioPciTransport<D>) -> Result<Self> {
        let function =
            Type0Function::new_with_configuration_space(dev.configuration_space.clone(), dev)
                .unwrap();
        Ok(VirtioPciDev { function })
    }
}

impl<D> Device for VirtioPciDev<D>
where
    D: VirtioPciDevice,
{
    fn name(&self) -> String {
        "virtio pci dev".to_string()
    }

    fn pause(&self) -> std::result::Result<(), DeviceSnapshotError> {
        self.function.pause()
    }

    fn resume(&self) -> std::result::Result<(), DeviceSnapshotError> {
        self.function.resume()
    }

    fn save(&self, writer: &mut dyn Write) -> std::result::Result<(), DeviceSnapshotError> {
        self.function.save(writer)
    }

    fn load(&mut self, reader: &mut dyn Read) -> std::result::Result<(), DeviceSnapshotError> {
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

    fn into_virtio_pci_device(
        self,
        irq_allocator: &mut IrqAllocator,
        tokio_runtime: Handle,
        memory: Arc<MemoryAddressSpace>,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<VirtioPciTransport<Self>> {
        let dev = VirtioPciTransport::new(
            irq_allocator,
            tokio_runtime,
            memory,
            irq_chip,
            VirtioTransportCommon::new(self)?,
        );
        Ok(dev)
    }

    fn into_pci_device(
        self,
        irq_allocator: &mut IrqAllocator,
        tokio_runtime: Handle,
        memory: Arc<MemoryAddressSpace>,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<VirtioPciDev<Self>> {
        let virtio_dev =
            self.into_virtio_pci_device(irq_allocator, tokio_runtime, memory, irq_chip)?;

        VirtioPciDev::try_from(virtio_dev)
    }
}
