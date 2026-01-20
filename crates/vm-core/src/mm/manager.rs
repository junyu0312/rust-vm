use std::collections::BTreeMap;
use std::collections::btree_map;

use anyhow::anyhow;
use anyhow::bail;

use crate::mm::allocator::MemoryContainer;
use crate::mm::region::MemoryRegion;

pub struct MemoryAddressSpace<C> {
    regions: BTreeMap<u64, MemoryRegion<C>>,
}

impl<C> Default for MemoryAddressSpace<C> {
    fn default() -> Self {
        Self {
            regions: Default::default(),
        }
    }
}

impl<'a, C> IntoIterator for &'a MemoryAddressSpace<C> {
    type Item = &'a MemoryRegion<C>;
    type IntoIter = btree_map::Values<'a, u64, MemoryRegion<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.regions.values()
    }
}

impl<'a, C> IntoIterator for &'a mut MemoryAddressSpace<C> {
    type Item = &'a mut MemoryRegion<C>;
    type IntoIter = btree_map::ValuesMut<'a, u64, MemoryRegion<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.regions.values_mut()
    }
}

impl<C> MemoryAddressSpace<C>
where
    C: MemoryContainer,
{
    fn is_overlapping(&self, region: &MemoryRegion<C>) -> bool {
        let new_left = region.gpa;
        let new_right = region.gpa + region.len as u64;

        for r in self.regions.values() {
            let left = r.gpa;
            let right = r.gpa + r.len as u64;

            if new_left < right && left < new_right {
                return true;
            }
        }

        false
    }

    pub fn try_insert(&mut self, region: MemoryRegion<C>) -> Result<(), MemoryRegion<C>> {
        if self.is_overlapping(&region) {
            return Err(region);
        }

        self.regions.insert(region.gpa, region);

        Ok(())
    }

    pub fn get_mut_by_gpa(&mut self, gpa: u64) -> Option<&mut MemoryRegion<C>> {
        self.regions
            .values_mut()
            .find(|region| gpa >= region.gpa && gpa < region.gpa + region.len as u64)
            .map(|v| v as _)
    }

    pub fn gpa_to_hva(&mut self, gpa: u64) -> anyhow::Result<*mut u8> {
        let region = self
            .get_mut_by_gpa(gpa)
            .ok_or_else(|| anyhow!("Memory region not found"))?;

        let offset = gpa - region.gpa;
        let hva = region
            .to_hva()
            .ok_or_else(|| anyhow!("memory is not initialied"))?;

        unsafe { Ok(hva.add(offset as usize)) }
    }

    pub fn memset(&mut self, gpa: u64, v: u8, len: usize) -> anyhow::Result<()> {
        let region = self
            .get_mut_by_gpa(gpa)
            .ok_or_else(|| anyhow!("Memory region not found"))?;

        let hva = region
            .to_hva()
            .ok_or_else(|| anyhow!("memory is not initialied"))?;

        let offset = gpa - region.gpa;
        if offset + len as u64 > region.len as u64 {
            bail!("Copy exceeds memory region bounds");
        }

        for i in 0..len {
            unsafe {
                *hva.add(i) = v;
            }
        }

        Ok(())
    }

    pub fn copy_from_slice(&mut self, gpa: u64, buf: &[u8], len: usize) -> anyhow::Result<()> {
        let region = self
            .get_mut_by_gpa(gpa)
            .ok_or_else(|| anyhow!("Memory region not found"))?;

        let hva = region
            .to_hva()
            .ok_or_else(|| anyhow!("memory is not initialied"))?;

        let offset = gpa - region.gpa;
        if offset + len as u64 > region.len as u64 {
            bail!("Copy exceeds memory region bounds");
        }

        unsafe {
            hva.add(offset as usize).copy_from(buf.as_ptr(), len);
        }

        Ok(())
    }
}
