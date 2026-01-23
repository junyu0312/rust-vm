use crate::arch::Arch;

pub struct AArch64;

impl Arch for AArch64 {
    const BASE_ADDRESS: u64 = 0x8000_0000;
}
