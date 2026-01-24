#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub mod vm_exit;

pub trait Arch {
    type VmExitReason;

    const BASE_ADDRESS: u64;
    const MMIO_START: u64;
    const MMIO_LEN: usize;
}
