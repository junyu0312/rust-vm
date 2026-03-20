use crate::utils::address_space::AddressSpace;
use crate::utils::address_space::Range;

pub type MmioRange = Range<u64>;

#[derive(Default)]
pub struct MmioLayout {
    address_space: AddressSpace<u64, ()>,
}

impl MmioLayout {
    pub fn new(mmio_start: u64, mmio_len: usize) -> Self {
        let mut address_space = AddressSpace::default();
        address_space
            .try_insert(
                Range {
                    start: mmio_start,
                    len: mmio_len,
                },
                (),
            )
            .unwrap();
        MmioLayout { address_space }
    }

    pub fn includes(&self, range: Range<u64>) -> bool {
        self.address_space.is_overlap(range.start, range.len)
    }

    pub fn in_mmio_region(&self, addr: u64) -> bool {
        self.address_space.try_get_value_by_key(addr).is_some()
    }
}
