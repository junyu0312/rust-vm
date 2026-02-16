use vm_pci::device::function::type0::Type0Function;
use vm_pci::device::pci_device::PciDevice;

use crate::device::VirtIoDevice;
use crate::transport::pci::VirtIoPciFunction;

pub trait VirtIoPciDevice: VirtIoDevice {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize;
    const CLASS_CODE: u32;
    const IRQ_PIN: u8;

    fn into_pci_device(self) -> PciDevice {
        let virtio_function = VirtIoPciFunction {
            transport: self.into(),
        };
        let function = Type0Function::new(virtio_function).unwrap();
        PciDevice::from_single_function(Box::new(function))
    }
}
