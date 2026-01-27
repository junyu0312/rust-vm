use crate::layout::MemoryLayout;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub mod vm_exit;

pub trait Arch {
    type VmExitReason;
    type Layout: MemoryLayout;

    fn get_layout(&self) -> &Self::Layout;

    fn get_layout_mut(&mut self) -> &mut Self::Layout;
}
