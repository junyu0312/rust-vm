use std::sync::Arc;

use vm_core::arch::irq::InterruptController;
use vm_core::device::mmio::layout::MmioRange;
use vm_core::device_manager::manager::DeviceManager;
use vm_core::monitor::MonitorServerBuilder;
use vm_device::device::Device;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioBalloonTranditional;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioMmioBalloonDevice;
use vm_device::device::virtio::virtio_balloon_traditional::monitor::VirtioBalloonMonitor;
use vm_device::device::virtio::virtio_blk::VirtioBlkDevice;
use vm_device::device::virtio::virtio_blk::VirtioMmioBlkDevice;
use vm_device::device::virtio::virtio_entropy::VirtioEntropy;
use vm_device::device::virtio::virtio_entropy::VirtioMmioEntropyDevice;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;
use vm_pci::root_complex::mmio::PciRootComplexMmio;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::pci::VirtioPciDevice;

use crate::error::Error;

pub trait InitDevice {
    fn init_devices<C>(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace<C>>,
        devices: Vec<Device>,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), Error>
    where
        C: MemoryContainer;
}

impl InitDevice for DeviceManager {
    fn init_devices<C>(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace<C>>,
        devices: Vec<Device>,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), Error>
    where
        C: MemoryContainer,
    {
        let pci_rc = PciRootComplexMmio::new(
            MmioRange {
                start: 0x1000_0000,
                len: 0x1000_0000,
            },
            0x2000_0000,
            0x1000_0000,
        );

        // TODO: Add cli
        {
            let virtio_pci_blk =
                VirtioBlkDevice::new(10, irq_chip.clone(), mm.clone()).into_pci_device();

            pci_rc
                .register_device(virtio_pci_blk)
                .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
        }

        // TODO: Add cli
        {
            let virtio_mmio_blk = VirtioMmioBlkDevice::new(
                VirtioDev::new(VirtioBlkDevice::new(2, irq_chip.clone(), mm.clone())),
                MmioRange {
                    start: 0x0900_1000,
                    len: 0x1000,
                },
            );
            self.register_mmio_device(Box::new(virtio_mmio_blk))?;
        }

        #[cfg(target_arch = "aarch64")]
        {
            use vm_device::device::pl011::Pl011;

            let pl011 = Pl011::<1>::new(
                MmioRange {
                    start: 0x0900_0000,
                    len: 0x1000,
                },
                irq_chip.clone(),
            );
            self.register_mmio_device(Box::new(pl011))?;
        }

        for device in devices {
            match device {
                Device::GicV3 => (),
                Device::VirtioMmioBalloon => {
                    let dev = VirtioDev::new(VirtioBalloonTranditional::new(
                        3,
                        irq_chip.clone(),
                        mm.clone(),
                    ));

                    let monitor = VirtioBalloonMonitor::new(dev.clone());
                    monitor_server_builder
                        .register_command_handler("balloon", Box::new(monitor))?;

                    // TODO: use mmio allocator?
                    let virtio_mmio_balloon = VirtioMmioBalloonDevice::new(
                        dev,
                        MmioRange {
                            start: 0x0900_2000,
                            len: 0x1000,
                        },
                    );
                    self.register_mmio_device(Box::new(virtio_mmio_balloon))?;
                }
                Device::VirtioMmioEntropy => {
                    let virtio_entropy = VirtioMmioEntropyDevice::new(
                        VirtioDev::new(VirtioEntropy::new(4, irq_chip.clone(), mm.clone())),
                        MmioRange {
                            start: 0x0900_3000,
                            len: 0x1000,
                        },
                    );

                    self.register_mmio_device(Box::new(virtio_entropy))?;
                }
                Device::VirtioPciEntropy => {
                    let virtio_entropy =
                        VirtioEntropy::new(11, irq_chip.clone(), mm.clone()).into_pci_device();

                    pci_rc
                        .register_device(virtio_entropy)
                        .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
                }
            }
        }

        self.register_mmio_device(Box::new(pci_rc))?;

        Ok(())
    }
}
