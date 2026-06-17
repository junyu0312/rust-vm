#[cfg(target_os = "linux")]
use std::cell::OnceCell;
use std::sync::Arc;

#[cfg(target_arch = "aarch64")]
use vm_core::arch::aarch64::layout::*;
use vm_core::arch::irq::InterruptController;
#[cfg(target_arch = "x86_64")]
use vm_core::arch::x86_64::layout::*;
use vm_core::virtualization::irq_allocator::IrqAllocator;
use vm_device::device::Device;
use vm_device::device::VfioTransport;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioBalloonTranditional;
use vm_device::device::virtio::virtio_balloon_traditional::monitor::VirtioBalloonMonitor;
use vm_device::device::virtio::virtio_blk::VirtioBlkDevice;
use vm_device::device::virtio::virtio_entropy::VirtioEntropy;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::root_complex_device::PciRootComplexDevice;
use vm_utils::range_allocator::RangeAllocator;
#[cfg(target_os = "linux")]
use vm_vfio::vfio::container::VfioContainer;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::mmio::VirtioMmioDevice;
use vm_virtio::transport::pci::VirtioPciDevice;

use crate::device::device_manager_v2::DeviceManagerV2;
use crate::device::error::InitDeviceError;
use crate::service::monitor::builder::MonitorServerBuilder;
#[cfg(target_arch = "aarch64")]
use crate::vm::device_builder::arch::aarch64::mmio_allocator;
#[cfg(target_arch = "x86_64")]
use crate::vm::device_builder::arch::x86_64::mmio_allocator;
#[cfg(target_arch = "x86_64")]
use crate::vm::device_builder::arch::x86_64::pio_allocator;

mod arch;
#[cfg(target_os = "linux")]
mod vfio;

pub struct DeviceManagerBuilder<'a> {
    irq_allocator: IrqAllocator,
    irq_chip: Arc<dyn InterruptController>,
    memory: Arc<MemoryAddressSpace>,
    monitor_server_builder: &'a mut MonitorServerBuilder,

    device_manager: DeviceManagerV2,

    #[cfg(target_os = "linux")]
    vfio_container: OnceCell<VfioContainer>,
    #[cfg(target_os = "linux")]
    need_dma_map: bool,

    #[cfg(target_arch = "x86_64")]
    pio_allocator: RangeAllocator<u16>,
    mmio_allocator: RangeAllocator<u64>,
    virtio_mmio_index_allocator: RangeAllocator<u8>,
}

impl<'a> DeviceManagerBuilder<'a> {
    fn alloc_irq(&mut self) -> Result<u32, InitDeviceError> {
        let irq = self
            .irq_allocator
            .alloc()
            .map_err(|err| InitDeviceError::AllocResource(Box::new(err)))?;

        Ok(irq)
    }

    fn init_device(
        &mut self,
        pci_root_complex: &mut PciRootComplexDevice,
        device: &Device,
    ) -> Result<(), InitDeviceError> {
        match device {
            Device::GicV3 => todo!(),
            Device::VirtioBlk { transport } => {
                let dev = VirtioBlkDevice::new(
                    self.alloc_irq()?,
                    self.irq_chip.clone(),
                    self.memory.clone(),
                );

                match transport {
                    VfioTransport::Mmio => {
                        self.device_manager
                            .attach_device(Box::new(dev.into_mmio_device(
                                &mut self.mmio_allocator,
                                &mut self.virtio_mmio_index_allocator,
                            )?))?;
                    }
                    VfioTransport::Pci => {
                        pci_root_complex
                            .register_device(Box::new(dev.into_pci_device()))
                            .map_err(|_| InitDeviceError::RegisterPciDevice)?;
                    }
                }
            }
            Device::VirtioBalloon { transport } => {
                let dev = VirtioDev::new(VirtioBalloonTranditional::new(
                    self.alloc_irq()?,
                    self.irq_chip.clone(),
                    self.memory.clone(),
                ));

                let monitor = VirtioBalloonMonitor::new(dev.clone());
                self.monitor_server_builder
                    .register_command_handler("balloon", Box::new(monitor))
                    .map_err(|_| InitDeviceError::RegisterMonitorCommand {
                        device: "balloon".to_string(),
                    })?;

                match transport {
                    VfioTransport::Mmio => {
                        todo!()
                    }
                    VfioTransport::Pci => {
                        todo!()
                    }
                }
            }
            Device::VirtioEntropy { transport } => {
                let dev = VirtioEntropy::new(
                    self.alloc_irq()?,
                    self.irq_chip.clone(),
                    self.memory.clone(),
                );

                match transport {
                    VfioTransport::Mmio => {
                        self.device_manager
                            .attach_device(Box::new(dev.into_mmio_device(
                                &mut self.mmio_allocator,
                                &mut self.virtio_mmio_index_allocator,
                            )?))?;
                    }
                    VfioTransport::Pci => {
                        pci_root_complex
                            .register_device(Box::new(dev.into_pci_device()))
                            .map_err(|_| InitDeviceError::RegisterPciDevice)?;
                    }
                }
            }
            #[cfg(target_os = "linux")]
            Device::VfioPci { name, path } => {
                let vfio_deivce = self.init_vfio_device(name.to_string(), path)?;

                self.need_dma_map = true;

                pci_root_complex
                    .register_device(Box::new(vfio_deivce))
                    .map_err(|_| {
                        InitDeviceError::PciDevice(vm_pci::error::Error::FailedRegisterPciDevice)
                    })?;
            }
        }

        Ok(())
    }

    fn init_pci_root_complex(&mut self) -> Result<PciRootComplexDevice, InitDeviceError> {
        Ok(PciRootComplexDevice::new(
            #[cfg(target_arch = "x86_64")]
            &mut self.pio_allocator,
            &mut self.mmio_allocator,
            #[cfg(target_arch = "x86_64")]
            (PCI_IO_PORT_WINDOW_START..PCI_IO_PORT_WINDOW_START + PCI_IO_PORT_WINDOW_LENGTH),
            ECAM_BASE as u64..ECAM_BASE as u64 + ECAM_LENGTH as u64,
            PCI_BAR_MMIO_WINDOW_START as u64
                ..PCI_BAR_MMIO_WINDOW_START as u64 + PCI_BAR_MMIO_WINDOW_LENGTH as u64,
        )?)
    }

    pub fn new(
        irq_chip: Arc<dyn InterruptController>,
        irq_allocator: IrqAllocator,
        memory: Arc<MemoryAddressSpace>,
        monitor_server_builder: &'a mut MonitorServerBuilder,
    ) -> Result<Self, InitDeviceError> {
        let device_manager = DeviceManagerV2::default();

        let mut virtio_mmio_index_allocator = RangeAllocator::<u8>::default();
        virtio_mmio_index_allocator.insert(0, 128).unwrap();

        Ok(DeviceManagerBuilder {
            irq_allocator,
            irq_chip,
            memory,
            monitor_server_builder,
            device_manager,

            #[cfg(target_os = "linux")]
            vfio_container: Default::default(),
            #[cfg(target_os = "linux")]
            need_dma_map: false,

            #[cfg(target_arch = "x86_64")]
            pio_allocator: pio_allocator(),
            mmio_allocator: mmio_allocator(),
            virtio_mmio_index_allocator,
        })
    }

    pub fn build(mut self, devices: &[Device]) -> Result<DeviceManagerV2, InitDeviceError> {
        #[cfg(target_os = "linux")]
        self.init_vfio()?;

        let mut pci_root_complex = self.init_pci_root_complex()?;

        self.init_device_arch()?;

        for device in devices {
            self.init_device(&mut pci_root_complex, device)?;
        }

        {
            let pci_root_complex = Box::new(pci_root_complex);
            self.device_manager.attach_device(pci_root_complex)?;
        }

        #[cfg(target_os = "linux")]
        self.vfio_dma_map()?;

        Ok(self.device_manager)
    }
}
