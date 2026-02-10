use zerocopy::FromBytes;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

#[derive(FromBytes, IntoBytes, KnownLayout)]
#[repr(C, packed)]
pub struct MsixCap {
    cap: u8,
    next: u8,
    pub ctrl: u16,
    pub table_offset: u32,
    pub pba_offset: u32,
}
