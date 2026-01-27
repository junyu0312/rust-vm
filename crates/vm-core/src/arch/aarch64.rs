use crate::arch::Arch;
use crate::arch::vm_exit::aarch64::VmExitReason;
use crate::layout::aarch64::AArch64Layout;

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
