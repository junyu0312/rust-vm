use vm_pci::device::capability::msix::MsixEntry;

#[derive(Default)]
pub struct VfioMsixEntry {
    pub entry: MsixEntry,
}

pub struct VfioMsix {
    pub table: Vec<MsixEntry>,
    pub table_bar: u8,
    pub table_offset: u32,
    pub pba: Vec<u8>,
    pub pba_bar: u8,
    pub pba_offset: u32,
}

impl VfioMsix {
    pub fn vectors(&self) -> u16 {
        self.table.len().try_into().unwrap()
    }
}
