use vm_pci::device::capability::msix::MsixEntry;
use vmm_sys_util::eventfd::EventFd;

pub struct VfioMsixInfo {
    pub event_fds: Vec<EventFd>,
    pub table_bar: u8,
    pub table_offset: u32,
    pub table_len: usize,
    pub pba_bar: u8,
    pub pba_offset: u32,
    pub pba_len: usize,
    pub cap_offset: u16,
    pub cap_len: u16,
}

#[derive(Default)]
pub struct VfioMsixEntry {
    pub entry: MsixEntry,
}

pub struct VfioMsix {
    pub table: Vec<MsixEntry>,
    pub pba: Vec<u8>,
    pub enabled: bool,
}

impl VfioMsix {
    pub fn vectors(&self) -> u16 {
        self.table.len().try_into().unwrap()
    }
}
