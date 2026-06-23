use crate::vfio_pci::interrupt::intx::VfioIntx;
use crate::vfio_pci::interrupt::msi::VfioMsi;
use crate::vfio_pci::interrupt::msix::VfioMsix;

pub(crate) mod intx;
pub(crate) mod msi;
pub(crate) mod msix;

#[derive(Default)]
pub struct VfioInterruptManager {
    pub intx: Option<VfioIntx>,
    pub msi: Option<VfioMsi>,
    pub msix: Option<VfioMsix>,
}
