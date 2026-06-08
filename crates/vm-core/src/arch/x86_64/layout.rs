use static_assertions::const_assert;

pub const RAM_BASE: u64 = 0x0000_0000;
pub const GDT_START: u32 = 0x0000_0500;
pub const BOOT_PARAMS_START: u32 = 0x0000_7000;
pub const CMDLINE_START: u32 = 0x0001_0000;
pub const ACPI_RSDT_START: u32 = 0x000e_0000;
pub const KERNEL_START: u32 = 0x0010_0000;
pub const INITRD_START: u32 = 0x1000_0000;

// For mmio devices (excluding pci devices)
pub const MMIO_START: u32 = 0xc000_0000;
pub const MMIO_LEN: u32 = 0x1000_0000;

pub const PCI_BAR_MMIO_WINDOW_START: u32 = 0xd000_0000;
pub const PCI_BAR_MMIO_WINDOW_LENGTH: u32 = 0x1000_0000;

pub const ECAM_BASE: u32 = 0xe000_0000;
// Ecam base, we only support one bus yet (see acpi),
// hence we reserved (32 devices * 8 functions) * 4K = 0x0010_0000
pub const ECAM_LENGTH: u32 = 0x0010_0000;

pub const IOAPIC_ADDR: u32 = 0xfec0_0000;
pub const APIC_ADDR: u32 = 0xfee0_0000;

const_assert!(PCI_BAR_MMIO_WINDOW_START >= MMIO_START + MMIO_LEN);
const_assert!(ECAM_BASE >= PCI_BAR_MMIO_WINDOW_START + PCI_BAR_MMIO_WINDOW_LENGTH);
const_assert!(IOAPIC_ADDR >= ECAM_BASE + ECAM_LENGTH);
