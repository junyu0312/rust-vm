use std::slice::Iter;
use std::slice::IterMut;
use std::sync::Arc;
use std::sync::Mutex;

use vm_core::arch::irq::InterruptController;
use vm_core::device::mmio::layout::MmioLayout;
use vm_core::device::mmio::layout::MmioRange;
use vm_core::device::mmio::mmio_device::MmioDevice;
use vm_core::device::pio::pio_device::PioDevice;
use vm_core::utils::address_space::AddressSpaceError;
use vm_core::utils::address_space::Range;
use vm_device::device::Device;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioBalloonTranditional;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioMmioBalloonDevice;
use vm_device::device::virtio::virtio_balloon_traditional::monitor::VirtioBalloonMonitor;
use vm_device::device::virtio::virtio_entropy::VirtioEntropy;
use vm_device::device::virtio::virtio_entropy::VirtioMmioEntropyDevice;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::root_complex::mmio::PciRootComplexMmio;
use vm_pci::root_complex::pci_root_complex::PciRootComplex;
use vm_utils::range_allocator::RangeAllocator;
#[cfg(target_os = "linux")]
use vm_vfio::vfio::container::VfioContainer;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::pci::VirtioPciDevice;

use crate::device::device_manager::irq_allocation::IrqAllocation;
use crate::device::device_manager::mmio::MmioAddressSpaceManager;
use crate::device::device_manager::pio::PioAddressSpaceManager;
use crate::device::error::InitDeviceError;
use crate::service::monitor::builder::MonitorServerBuilder;

pub(crate) mod snapshot;

mod arch;
mod irq_allocation;
mod mmio;
mod pci;
mod pio;
#[cfg(target_os = "linux")]
mod vfio;

pub struct DeviceManager {
    mmio_allocator: RangeAllocator<u64>,
    pub pio_manager: PioAddressSpaceManager,
    pub mmio_manager: MmioAddressSpaceManager,
    #[cfg(target_os = "linux")]
    vfio_container: Option<VfioContainer>,
}

impl DeviceManager {
    /// mmio_allocator: Range of mmio, excludes ECAM and Pci config space window
    pub fn new(mmio_layout: MmioLayout, mmio_allocator: RangeAllocator<u64>) -> Self {
        DeviceManager {
            mmio_allocator,
            pio_manager: PioAddressSpaceManager::default(),
            mmio_manager: MmioAddressSpaceManager::new(mmio_layout),
            #[cfg(target_os = "linux")]
            vfio_container: None,
        }
    }

    #[allow(dead_code)]
    pub fn register_pio_device(
        &mut self,
        device: Box<dyn PioDevice>,
    ) -> Result<(), AddressSpaceError> {
        self.pio_manager.register(device)
    }

    pub fn register_mmio_device(
        &mut self,
        device: Box<dyn MmioDevice>,
    ) -> Result<(), AddressSpaceError> {
        self.mmio_manager.register(device)
    }

    pub fn mmio_devices(&self) -> Iter<'_, Box<dyn MmioDevice>> {
        self.mmio_manager.devices()
    }

    pub fn mmio_devices_mut(&mut self) -> IterMut<'_, Box<dyn MmioDevice>> {
        self.mmio_manager.devices_mut()
    }

    fn init_device(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        irq_allocator: &mut IrqAllocation,
        irq_chip: Arc<dyn InterruptController>,
        mm: Arc<MemoryAddressSpace>,
        pci_root_complex: &mut PciRootComplex,
        device: &Device,
    ) -> Result<(), InitDeviceError> {
        match device {
            Device::GicV3 => todo!(),
            Device::VirtioMmioBalloon => {
                let dev = VirtioDev::new(VirtioBalloonTranditional::new(
                    irq_allocator.alloc(),
                    irq_chip.clone(),
                    mm.clone(),
                ));

                let monitor = VirtioBalloonMonitor::new(dev.clone());
                monitor_server_builder
                    .register_command_handler("balloon", Box::new(monitor))
                    .map_err(|_| InitDeviceError::RegisterMonitorCommand {
                        device: "balloon".to_string(),
                    })?;

                let virtio_mmio_balloon = VirtioMmioBalloonDevice::new(
                    dev,
                    MmioRange {
                        start: self
                            .mmio_allocator
                            .alloc(0x1000)
                            .map_err(|_| InitDeviceError::AllocMmioRange)?,
                        len: 0x1000,
                    },
                );
                self.register_mmio_device(Box::new(virtio_mmio_balloon))?;
            }
            Device::VirtioMmioEntropy => {
                let virtio_entropy = VirtioMmioEntropyDevice::new(
                    VirtioDev::new(VirtioEntropy::new(
                        irq_allocator.alloc(),
                        irq_chip.clone(),
                        mm.clone(),
                    )),
                    MmioRange {
                        start: self
                            .mmio_allocator
                            .alloc(0x1000)
                            .map_err(|_| InitDeviceError::AllocMmioRange)?,
                        len: 0x1000,
                    },
                );

                self.register_mmio_device(Box::new(virtio_entropy))?;
            }
            Device::VirtioPciEntropy => {
                let virtio_entropy =
                    VirtioEntropy::new(irq_allocator.alloc(), irq_chip.clone(), mm.clone())
                        .into_pci_device();

                pci_root_complex
                    .register_device(Box::new(virtio_entropy))
                    .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
            }
            #[cfg(target_os = "linux")]
            Device::VfioPci { name, path } => {
                let vfio_container = &self
                    .vfio_container
                    .as_ref()
                    .ok_or(InitDeviceError::VfioNotSupport)?;
                let dev = self.init_vfio_device(name.to_string(), vfio_container, path)?;
                pci_root_complex
                    .register_device(Box::new(dev))
                    .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
            }
        }

        Ok(())
    }

    pub fn init(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace>,
        devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError> {
        let mut irq_allocator = IrqAllocation::new(0);

        #[cfg(target_os = "linux")]
        self.init_vfio()?;

        let mut pci_root_complex = self.init_pci_root_complex()?;

        self.init_arch(
            &mut irq_allocator,
            mm.clone(),
            irq_chip.clone(),
            &mut pci_root_complex,
        )?;

        for device in devices {
            self.init_device(
                monitor_server_builder,
                &mut irq_allocator,
                irq_chip.clone(),
                mm.clone(),
                &mut pci_root_complex,
                device,
            )?;
        }

        {
            let pci_root_complex = Arc::new(Mutex::new(pci_root_complex));

            #[cfg(target_arch = "aarch64")]
            {
                use vm_core::arch::aarch64::layout::ECAM_BASE;
                use vm_core::arch::aarch64::layout::ECAM_LENGTH;
                use vm_core::arch::aarch64::layout::PCI_BAR_MMIO_WINDOW_LENGTH;
                use vm_core::arch::aarch64::layout::PCI_BAR_MMIO_WINDOW_START;

                let rc = PciRootComplexMmio::new(
                    pci_root_complex,
                    Range {
                        start: ECAM_BASE as u64,
                        len: ECAM_LENGTH as usize,
                    },
                    PCI_BAR_MMIO_WINDOW_START as u64,
                    PCI_BAR_MMIO_WINDOW_LENGTH as usize,
                );

                self.mmio_manager.register(Box::new(rc))?;
            }

            #[cfg(target_arch = "x86_64")]
            {
                use vm_core::arch::x86_64::layout::ECAM_BASE;
                use vm_core::arch::x86_64::layout::ECAM_LENGTH;
                use vm_core::arch::x86_64::layout::PCI_BAR_MMIO_WINDOW_LENGTH;
                use vm_core::arch::x86_64::layout::PCI_BAR_MMIO_WINDOW_START;
                use vm_pci::root_complex::pio::PciRootComplexPio;

                let mmio = PciRootComplexMmio::new(
                    pci_root_complex.clone(),
                    Range {
                        start: ECAM_BASE as u64,
                        len: ECAM_LENGTH as usize,
                    },
                    PCI_BAR_MMIO_WINDOW_START as u64,
                    PCI_BAR_MMIO_WINDOW_LENGTH as usize,
                );

                let pio = PciRootComplexPio::new(pci_root_complex);

                self.mmio_manager.register(Box::new(mmio))?;
                self.pio_manager.register(Box::new(pio))?;
            }
        }

        Ok(())
    }
}
