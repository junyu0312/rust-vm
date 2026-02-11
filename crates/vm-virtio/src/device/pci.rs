use std::sync::Arc;
use std::sync::Mutex;

use vm_pci::device::PciDevice;
use vm_pci::types::function::type0::Type0Function;

use crate::device::VirtIoDevice;
use crate::transport::pci::VirtIoPciFunction;

pub trait VirtIoPciDevice: VirtIoDevice {
    const DEVICE_SPECIFICATION_CONFIGURATION_LEN: usize;
    const CLASS_CODE: u32;
    const IRQ_LINE: u8;
    const IRQ_PIN: u8;

    fn into_pci_device(self) -> PciDevice {
        let virtio_function = VirtIoPciFunction {
            transport: Arc::new(Mutex::new(self.into())),
        };
        let function = Type0Function::new(virtio_function);
        PciDevice::new(vec![Box::new(function)])
    }
}
