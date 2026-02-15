#[derive(Clone)]
pub struct StandardCapability {
    pub(crate) cap_id: u8,
    _next: u8,
    pub(crate) data: Vec<u8>,
}

impl StandardCapability {
    pub fn new(cap_id: u8, data: Vec<u8>) -> Self {
        StandardCapability {
            cap_id,
            _next: 0,
            data,
        }
    }

    pub fn cap_len(&self) -> usize {
        2 + self.data.len()
    }
}

#[derive(Clone)]
pub struct ExtendedCapability {
    pub(crate) cap_id: u16,
    pub(crate) next_or_ver: u16,
    pub(crate) data: Vec<u8>,
}

impl ExtendedCapability {
    pub fn new(cap_id: u16, cap_version: u16, data: Vec<u8>) -> Self {
        ExtendedCapability {
            cap_id,
            next_or_ver: cap_version,
            data,
        }
    }

    pub fn cap_len(&self) -> usize {
        4 + self.data.len()
    }
}
