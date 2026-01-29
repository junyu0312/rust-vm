use crate::arch::Arch;
use crate::arch::vm_exit::x86_64::VmExitReason;
use crate::layout::x86_64::X86_64Layout;

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
