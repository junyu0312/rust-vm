use crate::utils::address_space::AddressSpace;
use crate::utils::address_space::Range;

pub type MmioRange = Range<u64>;

#[derive(Default)]
pub struct MmioLayout {
    address_space: AddressSpace<u64, ()>,
}

impl MmioLayout {
    pub fn try_insert(&mut self, start: u64, len: usize) {
        self.address_space
            .try_insert(Range { start, len }, ())
            .unwrap();
    }

    pub fn includes(&self, range: Range<u64>) -> bool {
        self.address_space.is_overlap(range.start, range.len)
    }

    pub fn in_mmio_region(&self, addr: u64) -> bool {
        self.address_space.try_get_value_by_key(addr).is_some()
    }
}
