use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use vm_core::arch::irq::InterruptController;
use vm_core::device::device_manager::DeviceManager;
use vm_core::device::mmio::MmioRange;
use vm_device::device::Device;
use vm_device::device::virtio::virtio_balloon_traditional::VirtioBalloonApi;
use vm_device::device::virtio::virtio_balloon_traditional::VirtioBalloonTranditional;
use vm_device::device::virtio::virtio_balloon_traditional::VirtioMmioBalloonDevice;
use vm_device::device::virtio::virtio_blk::VirtioBlkDevice;
use vm_device::device::virtio::virtio_blk::VirtioMmioBlkDevice;
use vm_mm::manager::MemoryAddressSpace;
use vm_mm::memory_container::MemoryContainer;
use vm_pci::root_complex::mmio::PciRootComplexMmio;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::pci::VirtioPciDevice;

use crate::error::Error;

pub trait InitDevice {
    fn init_devices<C>(
        &mut self,
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
        mm: Arc<MemoryAddressSpace<C>>,
        devices: Vec<Device>,
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), Error>
    where
        C: MemoryContainer,
    {
        {
            let pci_rc = PciRootComplexMmio::new(
                MmioRange {
                    start: 0x1000_0000,
                    len: 0x1000_0000,
                },
                0x2000_0000,
                0x1000_0000,
            );

            {
                let virtio_pci_blk =
                    VirtioBlkDevice::new(10, irq_chip.clone(), mm.clone()).into_pci_device();

                pci_rc
                    .register_device(virtio_pci_blk)
                    .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
            }

            self.register_mmio_device(Box::new(pci_rc))?;
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

        for device in devices {
            match device {
                Device::GicV3 => (), // irq_chip is initialized already
                Device::VirtioMmioBalloon => {
                    let dev = VirtioDev::new(VirtioBalloonTranditional::new(
                        3,
                        irq_chip.clone(),
                        mm.clone(),
                    ));

                    if false {
                        std::thread::spawn({
                            let dev = dev.clone();
                            move || {
                                let mut i = 0;
                                loop {
                                    i += 1;

                                    {
                                        sleep(Duration::from_secs(5));

                                        let mut dev = dev.lock().unwrap();
                                        dev.update_num_pages(i);
                                    }

                                    {
                                        sleep(Duration::from_secs(5));

                                        let mut dev = dev.lock().unwrap();
                                        dev.update_num_pages(0);
                                    }
                                }
                            }
                        });
                    }

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
            }
        }

        Ok(())
    }
}
