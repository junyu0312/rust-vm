use std::sync::Arc;

use vm_core::arch::irq::InterruptController;
use vm_core::device::mmio::layout::MmioRange;
use vm_device::device::pl011::Pl011;
use vm_device::device::virtio::virtio_blk::VirtioBlkDevice;
use vm_device::device::virtio::virtio_blk::VirtioMmioBlkDevice;
use vm_mm::manager::MemoryAddressSpace;
use vm_pci::root_complex::pci_root_complex::PciRootComplex;
use vm_virtio::transport::VirtioDev;
use vm_virtio::transport::pci::VirtioPciDevice;

use crate::device::device_manager::DeviceManager;
use crate::device::device_manager::irq_allocation::IrqAllocation;
use crate::device::error::InitDeviceError;

impl DeviceManager {
    pub fn init_arch(
        &mut self,
        irq_allocation: &mut IrqAllocation,
        mm: Arc<MemoryAddressSpace>,
        irq_chip: Arc<dyn InterruptController>,
        pci_rc: &mut PciRootComplex,
    ) -> Result<(), InitDeviceError> {
        {
            let pl011 = Pl011::new(
                MmioRange {
                    start: self
                        .mmio_allocator
                        .alloc(0x1000)
                        .map_err(|_| InitDeviceError::AllocMmioRange)?,
                    len: 0x1000,
                },
                irq_allocation.alloc(),
                irq_chip.clone(),
            );
            self.register_mmio_device(Box::new(pl011))?;
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
                    start: self
                        .mmio_allocator
                        .alloc(0x1000)
                        .map_err(|_| InitDeviceError::AllocMmioRange)?,
                    len: 0x1000,
                },
            );
            self.register_mmio_device(Box::new(virtio_mmio_blk))?;
        }

        Ok(())
    }
}
