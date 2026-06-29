use std::ops::Range;

use vmm_sys_util::eventfd::EventFd;

pub struct VfioMsiInfo {
    pub event_fds: Vec<EventFd>,
    pub vectors: u8,
    pub cap_offset_range: Range<u16>,
}

pub struct VfioMsi {
    pub enabled: bool,
}
