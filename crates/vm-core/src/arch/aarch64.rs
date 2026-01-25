use crate::arch::Arch;
use crate::arch::vm_exit::aarch64::VmExitReason;

pub mod layout;
pub struct AArch64;

impl Arch for AArch64 {
    type VmExitReason = VmExitReason;
}
