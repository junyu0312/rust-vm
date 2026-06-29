use vmm_sys_util::eventfd::EventFd;

pub struct VfioMsiInfo {
    pub event_fds: Vec<EventFd>,
    pub vectors: u8,
}

pub struct VfioMsi {
    pub enabled: bool,
}
