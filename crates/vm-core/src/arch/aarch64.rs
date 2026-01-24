use crate::arch::Arch;
use crate::arch::vm_exit::aarch64::VmExitReason;

pub struct AArch64;

impl Arch for AArch64 {
    type VmExitReason = VmExitReason;

    const BASE_ADDRESS: u64 = 0x8000_0000;
    const MMIO_START: u64 = 0x0900_0000;
    const MMIO_LEN: usize = 0x0100_0000;
}
