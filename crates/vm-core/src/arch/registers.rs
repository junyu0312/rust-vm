#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::AArch64CoreRegisters as ArchCoreRegisters;
#[cfg(target_arch = "aarch64")]
pub use aarch64::AArch64Registers as ArchRegisters;

#[cfg(target_arch = "x86_64")]
pub use x86_64::X86_64CoreRegisters as ArchCoreRegisters;
#[cfg(target_arch = "x86_64")]
pub use x86_64::X86_64Registers as ArchRegisters;
