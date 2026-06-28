use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::device::capability::PciCapId;
use crate::types::configuration_space::capability::StandardCapability;

pub const PCI_MSIX_FLAGS_MASKALL: u16 = 0x4000; /* Mask all vectors for this function */
pub const PCI_MSIX_FLAGS_ENABLE: u16 = 0x8000; /* MSI-X enable */
pub const PCI_MSIX_FLAGS_QSIZE: u16 = 0x07ff; /* Table size */

pub const PCI_MSIX_TABLE_BIR: u32 = 0x00000007; /* BAR index */
pub const PCI_MSIX_TABLE_OFFSET: u32 = 0xfffffff8; /* Offset into specified BAR */

pub const PCI_MSIX_PBA_BIR: u32 = 0x00000007; /* BAR index */
pub const PCI_MSIX_PBA_OFFSET: u32 = 0xfffffff8; /* Offset into specified BAR */

pub const PCI_MSIX_ENTRY_CTRL_MASKBIT: u32 = 0x00000001;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct MsixEntry {
    pub addr_lo: u32,
    pub addr_hi: u32,
    pub data: u32,
    pub control: u32,
}

impl Default for MsixEntry {
    fn default() -> Self {
        Self {
            addr_lo: Default::default(),
            addr_hi: Default::default(),
            data: Default::default(),
            control: 1, // Masked as default?
        }
    }
}

impl MsixEntry {
    pub fn is_mask(&self) -> bool {
        self.control & PCI_MSIX_ENTRY_CTRL_MASKBIT != 0
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct PciMsixCap {
    pub cap: u8,
    next: u8,
    pub ctrl: u16,
    pub table_offset: u32,
    pub pba_offset: u32,
}

impl PciMsixCap {
    pub fn new(size: u16, table_bar: u8, table_offset: u32, pba_bar: u8, pba_offset: u32) -> Self {
        assert!(size > 0 && size <= 2048);
        let ctrl = size - 1;

        assert!(table_bar < 6);
        assert!(pba_bar < 6);

        PciMsixCap {
            cap: PciCapId::MsiX as u8,
            next: Default::default(),
            ctrl,
            table_offset: (table_offset << 3) | ((table_bar & 0x7) as u32),
            pba_offset: (pba_offset << 3) | ((pba_bar & 0x7) as u32),
        }
    }

    pub fn enable(&self) -> bool {
        self.ctrl & PCI_MSIX_FLAGS_ENABLE != 0
    }

    // Mask all vectors
    pub fn function_mask(&self) -> bool {
        self.ctrl & PCI_MSIX_FLAGS_MASKALL != 0
    }
}

impl From<PciMsixCap> for StandardCapability {
    fn from(cap: PciMsixCap) -> Self {
        StandardCapability::new(cap.cap, cap.as_bytes()[2..].into())
    }
}
