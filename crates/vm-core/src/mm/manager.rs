use std::collections::BTreeMap;
use std::collections::btree_map;

use anyhow::anyhow;
use anyhow::bail;

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

    pub fn gpa_to_ptr(&mut self, gpa: usize) -> anyhow::Result<*mut u8> {
        let region = self
            .get_mut_by_gpa(gpa)
            .ok_or_else(|| anyhow!("Memory region not found"))?;

        let offset = gpa - region.gpa;

        Ok(unsafe { region.ptr.add(offset) })
    }

    pub fn memset(&mut self, gpa: usize, v: u8, len: usize) -> anyhow::Result<()> {
        let region = self
            .get_mut_by_gpa(gpa)
            .ok_or_else(|| anyhow!("Memory region not found"))?;

        let offset = gpa - region.gpa;
        if offset + len > region.len {
            bail!("Copy exceeds memory region bounds");
        }

        for i in 0..len {
            unsafe {
                *region.ptr.add(i) = v;
            }
        }

        Ok(())
    }

    pub fn copy_from_slice(&mut self, gpa: usize, buf: &[u8], len: usize) -> anyhow::Result<()> {
        let region = self
            .get_mut_by_gpa(gpa)
            .ok_or_else(|| anyhow!("Memory region not found"))?;

        let offset = gpa - region.gpa;
        if offset + len > region.len {
            bail!("Copy exceeds memory region bounds");
        }

        unsafe {
            region.ptr.add(offset).copy_from(buf.as_ptr(), len);
        }

        Ok(())
    }
}
