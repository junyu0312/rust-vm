use static_assertions::const_assert;
use static_assertions::const_assert_eq;

pub const MMIO_START: u32 = 0x0900_0000;
pub const MMIO_LEN: u32 = 0x0700_0000;

pub const ECAM_BASE: u32 = 0x1000_0000;
pub const ECAM_LENGTH: u32 = 0x1000_0000;

pub const PCI_BAR_MMIO_WINDOW_START: u32 = 0x2000_0000;
pub const PCI_BAR_MMIO_WINDOW_LENGTH: u32 = 0x1000_0000;

pub const GIC_DISTRIBUTOR: u64 = 0x3000_0000;
pub const GIC_REDISTRIBUTOR: u64 = 0x3001_0000;
pub const GIC_MSI: u64 = 0x3a00_0000;
pub const RAM_BASE: u64 = 0x4000_0000;
pub const DTB_START: u64 = 0x4400_0000; // Reserve 64MB for kernel
pub const INITRD_START: u64 = 0x44200000; // DTB + 2MB

const KERNEL_MAX: usize = 0x0400_0000;

const_assert!(ECAM_BASE >= MMIO_START + MMIO_LEN);
const_assert!(PCI_BAR_MMIO_WINDOW_START >= ECAM_BASE + ECAM_LENGTH);
const_assert_eq!(RAM_BASE + KERNEL_MAX as u64, DTB_START);
