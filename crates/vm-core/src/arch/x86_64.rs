use crate::arch::Arch;
use crate::arch::x86_64::layout::X86_64Layout;
use crate::arch::x86_64::vm_exit::VmExitReason;

pub mod layout;
pub mod vcpu;
pub mod vm_exit;

pub const BASE_ADDRESS: u64 = 0x0;

pub struct X86_64;

impl Arch for X86_64 {
    type VmExitReason = VmExitReason;
    type Layout = X86_64Layout;

    fn get_layout(&self) -> &X86_64Layout {
        todo!()
    }

    fn get_layout_mut(&mut self) -> &mut X86_64Layout {
        todo!()
    }
}
