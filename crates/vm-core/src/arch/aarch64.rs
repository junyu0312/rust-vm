use crate::arch::Arch;
use crate::arch::aarch64::layout::AArch64Layout;
use crate::arch::aarch64::vm_exit::VmExitReason;

pub mod firmware;
pub mod irq;
pub mod layout;
pub mod vcpu;
pub mod vm_exit;

pub struct AArch64 {
    pub layout: AArch64Layout,
}

impl Arch for AArch64 {
    type VmExitReason = VmExitReason;
    type Layout = AArch64Layout;

    fn get_layout(&self) -> &AArch64Layout {
        &self.layout
    }

    fn get_layout_mut(&mut self) -> &mut AArch64Layout {
        &mut self.layout
    }
}
