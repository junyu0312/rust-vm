#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub mod vm_exit;

pub trait Arch {
    type VmExitReason;
}
