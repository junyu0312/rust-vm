const PSCI_0_2_FN_BASE: u32 = 0x84000000;
pub const fn psci_0_2_fn(i: u32) -> u32 {
    PSCI_0_2_FN_BASE + i
}
pub const PSCI_0_2_64BIT: u32 = 0x40000000;
pub const PSCI_0_2_FN64_BASE: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT;
pub const fn psci_0_2_fn64(i: u32) -> u32 {
    PSCI_0_2_FN64_BASE + i
}

pub struct PsciFunctionId(u32);

impl PsciFunctionId {
    pub fn is_64bit(&self) -> bool {
        (self.0 & PSCI_0_2_64BIT) != 0
    }
}
