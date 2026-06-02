use std::sync::Arc;

use vm_core::arch::irq::InterruptController;
use vm_core::device::mmio::layout::MmioRange;
use vm_core::device_manager::DeviceManager;
use vm_device::device::Device;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioBalloonTranditional;
use vm_device::device::virtio::virtio_balloon_traditional::device::VirtioMmioBalloonDevice;
use vm_device::device::virtio::virtio_balloon_traditional::monitor::VirtioBalloonMonitor;
use vm_device::device::virtio::virtio_blk::VirtioBlkDevice;
use vm_device::device::virtio::virtio_blk::VirtioMmioBlkDevice;
use vm_device::device::virtio::virtio_entropy::VirtioEntropy;
use vm_device::device::virtio::virtio_entropy::VirtioMmioEntropyDevice;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::root_complex::mmio::PciRootComplexMmio;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::pci::VirtioPciDevice;

use crate::device::error::InitDeviceError;
use crate::device::irq_allocation::IrqAllocation;
use crate::service::monitor::builder::MonitorServerBuilder;

pub(crate) mod error;

mod irq_allocation;

pub trait InitDevice {
    fn init_devices(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace>,
        devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError>;
}

impl InitDevice for DeviceManager {
    #[cfg(target_arch = "x86_64")]
    fn init_devices(
        &mut self,
        _monitor_server_builder: &mut MonitorServerBuilder,
        _mm: Arc<MemoryAddressSpace>,
        _devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError> {
        use vm_device::device::cmos::Cmos;
        use vm_device::device::dummy::Dummy;
        use vm_device::device::i8042::I8042;
        use vm_device::device::post_debug::PostDebug;
        use vm_device::device::uart8250::Uart8250;
        use vm_pci::root_complex::pio::PciRootComplexPio;

        let uart8250_com1 = Uart8250::<4>::new(Some(0x3f8), irq_chip.clone());
        let uart8250_com2 = Uart8250::<3>::new(Some(0x2f8), irq_chip.clone());
        let uart8250_com3 = Uart8250::<4>::new(Some(0x3e8), irq_chip.clone());
        let uart8250_com4 = Uart8250::<3>::new(Some(0x2e8), irq_chip.clone());
        let cmos = Cmos;
        let post_debug = PostDebug;
        let dummy = Dummy;
        let i8042 = I8042::new(irq_chip);
        let pci_rc = PciRootComplexPio::default();

        self.register_pio_device(Box::new(uart8250_com1))?;
        self.register_pio_device(Box::new(uart8250_com2))?;
        self.register_pio_device(Box::new(uart8250_com3))?;
        self.register_pio_device(Box::new(uart8250_com4))?;
        self.register_pio_device(Box::new(cmos))?;
        self.register_pio_device(Box::new(post_debug))?;
        self.register_pio_device(Box::new(dummy))?;
        self.register_pio_device(Box::new(i8042))?;
        self.register_pio_device(Box::new(pci_rc))?;

        Ok(())
    }

    #[cfg(target_arch = "aarch64")]
    fn init_devices(
        &mut self,
        monitor_server_builder: &mut MonitorServerBuilder,
        mm: Arc<MemoryAddressSpace>,
        devices: &[Device],
        irq_chip: Arc<dyn InterruptController>,
    ) -> Result<(), InitDeviceError> {
        let mut irq_allocation = IrqAllocation::new(0);

        let pci_rc = PciRootComplexMmio::new(
            MmioRange {
                start: 0x1000_0000,
                len: 0x1000_0000,
            },
            0x2000_0000,
            0x1000_0000,
        );

        #[cfg(target_arch = "aarch64")]
        {
            use vm_device::device::pl011::Pl011;

            let pl011 = Pl011::new(
                MmioRange {
                    start: 0x0900_0000,
                    len: 0x1000,
                },
                irq_allocation.alloc(),
                irq_chip.clone(),
            );
            self.register_mmio_device(Box::new(pl011))?;
        }

        for device in devices {
            match device {
                Device::GicV3 => (),
                Device::VirtioMmioBalloon => {
                    let dev = VirtioDev::new(VirtioBalloonTranditional::new(
                        irq_allocation.alloc(),
                        irq_chip.clone(),
                        mm.clone(),
                    ));

                    let monitor = VirtioBalloonMonitor::new(dev.clone());
                    monitor_server_builder
                        .register_command_handler("balloon", Box::new(monitor))
                        .map_err(|_| InitDeviceError::RegisterMonitorCommand {
                            device: "balloon".to_string(),
                        })?;

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
                        VirtioDev::new(VirtioEntropy::new(
                            irq_allocation.alloc(),
                            irq_chip.clone(),
                            mm.clone(),
                        )),
                        MmioRange {
                            start: 0x0900_3000,
                            len: 0x1000,
                        },
                    );

                    self.register_mmio_device(Box::new(virtio_entropy))?;
                }
                Device::VirtioPciEntropy => {
                    let virtio_entropy =
                        VirtioEntropy::new(irq_allocation.alloc(), irq_chip.clone(), mm.clone())
                            .into_pci_device();

                    pci_rc
                        .register_device(Box::new(virtio_entropy))
                        .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
                }
            }
        }

        // TODO: Add cli
        {
            let virtio_pci_blk =
                VirtioBlkDevice::new(irq_allocation.alloc(), irq_chip.clone(), mm.clone())
                    .into_pci_device();

            pci_rc
                .register_device(Box::new(virtio_pci_blk))
                .map_err(|_| vm_pci::error::Error::FailedRegisterPciDevice)?;
        }

        // TODO: Add cli
        {
            let virtio_mmio_blk = VirtioMmioBlkDevice::new(
                VirtioDev::new(VirtioBlkDevice::new(
                    irq_allocation.alloc(),
                    irq_chip.clone(),
                    mm.clone(),
                )),
                MmioRange {
                    start: 0x0900_1000,
                    len: 0x1000,
                },
            );
            self.register_mmio_device(Box::new(virtio_mmio_blk))?;
        }

        self.register_mmio_device(Box::new(pci_rc))?;

        Ok(())
    }
}
