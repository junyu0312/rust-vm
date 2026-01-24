use crate::arch::Arch;

pub struct AArch64;

impl Arch for AArch64 {
    const BASE_ADDRESS: u64 = 0x8000_0000;
    const MMIO_START: u64 = 0x0900_0000;
    const MMIO_LEN: usize = 0x0100_0000;
}
