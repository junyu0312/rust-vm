use anyhow::bail;

use crate::device::address_space::Range;

pub mod mmio_as_manager;
pub mod mmio_device;

pub type MmioRange = Range<u64>;

#[derive(Debug, Default)]
pub struct MmioLayout {
    address_space: Vec<MmioRange>,
}

impl MmioLayout {
    pub fn new(mmio_start: u64, mmio_len: usize) -> Self {
        MmioLayout {
            address_space: vec![MmioRange {
                start: mmio_start,
                len: mmio_len,
            }],
        }
    }

    fn is_overlap(&self, range: &MmioRange) -> bool {
        let left = range.start;
        let right = range.start + range.len as u64;

        self.address_space.iter().any(|r| {
            let old_left = r.start;
            let old_right = r.start + r.len as u64;

            left < old_right && right > old_left
        })
    }

    pub fn try_insert(&mut self, mmio_range: MmioRange) -> anyhow::Result<()> {
        if self.is_overlap(&mmio_range) {
            bail!("overlap");
        }

        self.address_space.push(mmio_range);

        Ok(())
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

    pub fn in_mmio_region(&self, addr: u64) -> bool {
        self.contains(addr)
    }
}
