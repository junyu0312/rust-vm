use std::ops::Range;

use vm_pci::device::capability::msix::MsixEntry;
use vmm_sys_util::eventfd::EventFd;

pub struct VfioMsixInfo {
    pub event_fds: Vec<EventFd>,
    pub table_bar: u8,
    pub table_offset: u32,
    pub table_len: u32,
    pub pba_bar: u8,
    pub pba_offset: u32,
    pub pba_len: u32,
    pub cap_offset_range: Range<u16>,
}

pub struct VfioMsix {
    pub table: Vec<MsixEntry>,
    pub _pba: Vec<u8>,
    pub enabled: bool,
}
