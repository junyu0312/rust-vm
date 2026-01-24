use crate::device::Range;

pub type MmioRange = Range<u64>;

pub struct MmioLayout {
    address_space: Vec<MmioRange>,
}

impl MmioLayout {
    pub fn new(start: u64, len: usize) -> Self {
        MmioLayout {
            address_space: vec![MmioRange { start, len }],
        }
    }

    pub fn contains(&self, addr: u64) -> bool {
        self.address_space
            .iter()
            .any(|s| addr >= s.start && addr < s.start + s.len as u64)
    }

    pub fn includes(&self, range: MmioRange) -> bool {
        self.address_space.iter().any(|s| {
            range.start >= s.start && range.start + range.len as u64 <= s.start + s.len as u64
        })
    }
}
