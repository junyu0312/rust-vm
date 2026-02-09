use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use vm_core::device::device_manager::DeviceManager;
use vm_core::device::mmio::MmioRange;
use vm_core::irq::InterruptController;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_device::device::dummy_pci::DummyPci;
use vm_device::device::pl011::Pl011;
use vm_device::device::virtio::virtio_blk::VirtIoBlkDevice;
use vm_device::device::virtio::virtio_blk::VirtIoMmioBlkDevice;
use vm_device::pci::device::PciDevice;
use vm_device::pci::root_complex::mmio::PciRootComplexMmio;
use vm_device::pci::types::function::type0::Type0Function;

pub fn init_device<C>(
    mm: Arc<Mutex<MemoryAddressSpace<C>>>,
    io_address_space: &mut DeviceManager,
    irq_chip: Arc<dyn InterruptController>,
) -> anyhow::Result<()>
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

        let function = Type0Function::new(DummyPci::default());
        let pci_device = PciDevice::new(vec![Box::new(function)]);

        pci_rc
            .register_device(pci_device)
            .map_err(|_| anyhow!("failed to register pci device"))?;
        io_address_space.register_mmio_device(Box::new(pci_rc))?;
    }

    {
        let pl011 = Pl011::<1>::new(
            MmioRange {
                start: 0x0900_0000,
                len: 0x1000,
            },
            irq_chip.clone(),
        );
        io_address_space.register_mmio_device(Box::new(pl011))?;
    }

    {
        let virtio_mmio_blk = VirtIoMmioBlkDevice::new(
            VirtIoBlkDevice::new(2, irq_chip, mm),
            MmioRange {
                start: 0x0900_1000,
                len: 0x1000,
            },
        );
        io_address_space.register_mmio_device(Box::new(virtio_mmio_blk))?;
    }

    // let virtio_mmio_kbd = VirtIOMmioKbd::<48, C>::new(
    //     mm,
    //     "virtio-mmio-kbd-01".to_string(),
    //     MmioRange {
    //         start: 0x0900_1000,
    //         len: 0x1000,
    //     },
    //     irq_chip,
    //     rx,
    // );
    // io_address_space.register(Box::new(virtio_mmio_kbd))?;

    Ok(())
}
