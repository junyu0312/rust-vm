#[cfg(target_arch = "aarch64")]
pub(crate) mod aarch64;

#[cfg(target_arch = "x86_64")]
pub(crate) mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::InterruptMapEntryAArch64 as InterruptMapEntry;

#[cfg(target_arch = "x86_64")]
pub use x86_64::InterruptMapEntryX86_64 as InterruptMapEntry;

pub trait InterruptMapEntryArch {
    fn to_vec(&self) -> Vec<u32>;
}
