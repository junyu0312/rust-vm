use crate::vfio_pci::interrupt::intx::VfioIntx;
use crate::vfio_pci::interrupt::intx::VfioIntxInfo;
use crate::vfio_pci::interrupt::msi::VfioMsi;
use crate::vfio_pci::interrupt::msi::VfioMsiInfo;
use crate::vfio_pci::interrupt::msix::VfioMsix;
use crate::vfio_pci::interrupt::msix::VfioMsixInfo;

pub(crate) mod intx;
pub(crate) mod msi;
pub(crate) mod msix;

// Invariant
#[derive(Default)]
pub struct VfioInterruptInfo {
    pub intx: Option<VfioIntxInfo>,
    pub msi: Option<VfioMsiInfo>,
    pub msix: Option<VfioMsixInfo>,
}

#[derive(Default)]
pub struct VfioInterruptManager {
    pub intx: Option<VfioIntx>,
    pub msi: Option<VfioMsi>,
    pub msix: Option<VfioMsix>,
}
