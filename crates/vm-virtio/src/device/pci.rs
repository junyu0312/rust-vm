use vm_mm::allocator::MemoryContainer;
use vm_pci::device::function::type0::Type0Function;
use vm_pci::device::pci_device::PciDevice;

use crate::device::VirtioDevice;
use crate::transport::VirtioDev;
use crate::transport::pci::VirtioPciFunction;

pub trait VirtioPciDevice<C>: VirtioDevice<C>
where
    C: MemoryContainer,
{
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize;
    const CLASS_CODE: u32;
    const IRQ_PIN: u8;

    fn into_pci_device(self) -> PciDevice {
        let virtio_function = VirtioPciFunction::<C, _> {
            dev: VirtioDev::new(self),
        };
        let function = Type0Function::new(virtio_function).unwrap();
        PciDevice::from_single_function(Box::new(function))
    }
}
