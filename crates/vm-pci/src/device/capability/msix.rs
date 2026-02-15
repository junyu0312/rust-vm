use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::device::capability::PciCapId;

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C, packed)]
pub struct PciMsixCap {
    cap: u8,
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
            table_offset: (table_offset << 3) | (table_bar as u32),
            pba_offset: (pba_offset << 3) | (pba_offset as u32),
        }
    }
}
