use crate::types::device::PciDevice;

pub(crate) mod pci_root_complex;

pub mod mmio;
pub mod pio;

mod mmio_router;

pub trait PciRootComplexOps {
    fn register_device(&self, device: Box<dyn PciDevice>) -> Result<(), Box<dyn PciDevice>>;
}
