use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use vm_core::device::device_manager::DeviceManager;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_core::layout::aarch64::GIC_DISTRIBUTOR;
use vm_core::layout::aarch64::GIC_REDISTRIBUTOR;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_device::device::Device;
use vm_device::device::virtio::virtio_blk::VirtIoBlkDevice;
use vm_device::device::virtio::virtio_blk::VirtIoMmioBlkDevice;
use vm_pci::root_complex::mmio::PciRootComplexMmio;
use vm_virtio::device::pci::VirtIoPciDevice;

pub trait InitDevice {
    fn init_devices<C>(
        &mut self,
        mm: Arc<Mutex<MemoryAddressSpace<C>>>,
        vcpus: usize,
        devices: Vec<Device>,
        irq_chip: Option<Arc<dyn InterruptController>>,
    ) -> anyhow::Result<()>
    where
        C: MemoryContainer;
}

impl InitDevice for DeviceManager {
    fn init_devices<C>(
        &mut self,
        mm: Arc<Mutex<MemoryAddressSpace<C>>>,
        vcpus: usize,
        devices: Vec<Device>,
        irq_chip: Option<Arc<dyn InterruptController>>,
    ) -> anyhow::Result<()>
    where
        C: MemoryContainer,
    {
        let irq_chip = match irq_chip {
            Some(irq_chip) => irq_chip,
            None => {
                let irq_chip = devices
                    .iter()
                    .find(|dev| dev.is_irq_chip())
                    .ok_or(anyhow!("irq_chip must be specified"))?;

                match irq_chip {
                    #[cfg(target_arch = "aarch64")]
                    Device::GicV3 => {
                        use vm_device::device::gic::{
                            gic_common::config::GicConfig, gic_v3::GicV3,
                        };

                        let gic_v3 = GicV3::new(GicConfig {
                            distributor_base: GIC_DISTRIBUTOR,
                            redistributor_base: GIC_REDISTRIBUTOR,
                            are: false,
                            mbis: true,
                            security_extn: false,
                            nmi: true,
                            extended_spi: false,
                            cpu_number: vcpus,
                            redist_stride: None,
                            vlpis: false,
                        });
                        self.register_mmio_device(gic_v3.get_device())?;
                        Arc::new(gic_v3)
                    }
                }
            }
        };

        self.register_irq_chip(irq_chip.clone())?;

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
                    VirtIoBlkDevice::new(10, irq_chip.clone(), mm.clone()).into_pci_device();

                pci_rc
                    .register_device(virtio_pci_blk)
                    .map_err(|_| anyhow!("failed to register pci device"))?;
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
            let virtio_mmio_blk = VirtIoMmioBlkDevice::new(
                VirtIoBlkDevice::new(2, irq_chip, mm),
                MmioRange {
                    start: 0x0900_1000,
                    len: 0x1000,
                },
            );
            self.register_mmio_device(Box::new(virtio_mmio_blk))?;
        }

        Ok(())
    }
}
