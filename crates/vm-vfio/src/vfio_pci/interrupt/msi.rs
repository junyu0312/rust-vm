use std::ops::Range;

use vmm_sys_util::eventfd::EventFd;

pub struct VfioMsiInfo {
    pub event_fds: Vec<EventFd>,
    pub bit64: bool,
    pub mask: bool,
    pub cap_offset_range: Range<u16>,
}

pub struct VfioMsi {
    pub gsi: Vec<Option<u32>>,
    pub irqrd: Vec<bool>,
    pub enabled: bool,
}
