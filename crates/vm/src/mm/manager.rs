use std::collections::BTreeMap;
use std::collections::btree_map;

use crate::mm::region::MemoryRegion;

#[derive(Default)]
pub struct MemoryRegions {
    regions: BTreeMap<usize, MemoryRegion>,
}

impl<'a> IntoIterator for &'a MemoryRegions {
    type Item = &'a MemoryRegion;
    type IntoIter = btree_map::Values<'a, usize, MemoryRegion>;

    fn into_iter(self) -> Self::IntoIter {
        self.regions.values()
    }
}

impl MemoryRegions {
    fn is_overlapping(&self, region: &MemoryRegion) -> bool {
        let new_left = region.gpa;
        let new_right = region.gpa + region.len;

        for r in self.regions.values() {
            let left = r.gpa;
            let right = r.gpa + r.len;

            if new_left < right && left < new_right {
                return true;
            }
        }

        false
    }

    pub fn try_insert(&mut self, region: MemoryRegion) -> Result<(), MemoryRegion> {
        if self.is_overlapping(&region) {
            return Err(region);
        }

        self.regions.insert(region.gpa, region);

        Ok(())
    }

    pub fn get_mut_by_gpa(&mut self, gpa: usize) -> Option<&mut MemoryRegion> {
        self.regions
            .values_mut()
            .find(|region| gpa >= region.gpa && gpa < region.gpa + region.len)
            .map(|v| v as _)
    }
}
