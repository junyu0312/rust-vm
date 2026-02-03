use std::collections::BTreeMap;
use std::collections::btree_map;

use crate::mm::Error;
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
    pub fn try_insert(&mut self, region: MemoryRegion<C>) -> Result<(), MemoryRegion<C>> {
        if self.is_overlapping(&region) {
            return Err(region);
        }

        self.regions.insert(region.gpa, region);

        Ok(())
    }

    pub fn gpa_to_hva(&mut self, gpa: u64) -> Result<*mut u8, Error> {
        let region = self.try_get_region_by_gpa_mut(gpa)?;
        let hva = region.try_to_hva()?;

        let offset = gpa - region.gpa;

        unsafe { Ok(hva.add(offset as usize)) }
    }

    pub fn memset(&mut self, gpa: u64, val: u8, len: usize) -> Result<(), Error> {
        let region = self.try_get_region_by_gpa_mut(gpa)?;
        let hva = region.try_to_hva()?;
        let offset = gpa - region.gpa;

        if offset + len as u64 > region.len as u64 {
            return Err(Error::MemoryOverflow);
        }

        unsafe { hva.add(offset as usize).write_bytes(val, len) };

        Ok(())
    }

    pub fn copy_from_slice(&mut self, gpa: u64, buf: &[u8], len: usize) -> Result<(), Error> {
        let region = self.try_get_region_by_gpa_mut(gpa)?;
        let hva = region.try_to_hva()?;
        let offset = gpa - region.gpa;

        if offset + len as u64 > region.len as u64 {
            return Err(Error::MemoryOverflow);
        }

        unsafe {
            hva.add(offset as usize).copy_from(buf.as_ptr(), len);
        }

        Ok(())
    }

    fn is_overlapping(&self, region: &MemoryRegion<C>) -> bool {
        let new_left = region.gpa;
        let new_right = region.gpa + region.len as u64;

        self.regions.values().any(|r| {
            let left = r.gpa;
            let right = left + r.len as u64;
            new_left < right && left < new_right
        })
    }

    fn get_mut_by_gpa(&mut self, gpa: u64) -> Option<&mut MemoryRegion<C>> {
        self.regions
            .values_mut()
            .find(|region| gpa >= region.gpa && gpa < region.gpa + region.len as u64)
            .map(|v| v as _)
    }

    fn try_get_region_by_gpa_mut(&mut self, gpa: u64) -> Result<&mut MemoryRegion<C>, Error> {
        self.get_mut_by_gpa(gpa).ok_or(Error::AccessInvalidGpa(gpa))
    }
}

#[cfg(test)]
mod tests {
    use memmap2::MmapMut;

    use crate::mm::allocator::mmap_allocator::MmapAllocator;
    use crate::mm::manager::MemoryAddressSpace;
    use crate::mm::region::MemoryRegion;

    #[test]
    fn test_memory_layout() -> anyhow::Result<()> {
        let mut memory_as = MemoryAddressSpace::<MmapMut>::default();

        assert!(
            memory_as
                .try_insert(MemoryRegion::placeholder(0, 10))
                .is_ok()
        );
        assert!(
            memory_as
                .try_insert(MemoryRegion::placeholder(5, 10))
                .is_err()
        );
        assert!(
            memory_as
                .try_insert(MemoryRegion::placeholder(10, 10))
                .is_ok()
        );

        Ok(())
    }

    #[test]
    fn test_memory_write() -> anyhow::Result<()> {
        const GPA: u64 = 0;
        const LEN: usize = 512;
        let mut memory_as = MemoryAddressSpace::<MmapMut>::default();

        let mut region = MemoryRegion::placeholder(GPA, LEN);
        region.alloc(&MmapAllocator)?;
        let hva = region.try_to_hva()?;

        assert!(memory_as.try_insert(region).is_ok());

        {
            // Test gpa_to_hva ok
            let addr = memory_as.gpa_to_hva(0)?;
            assert_eq!(addr, hva);
        }

        {
            // Test invalid gpa
            assert!(memory_as.gpa_to_hva(LEN as u64).is_err());
        }

        {
            // Test memset ok
            let val = 0xcd;
            memory_as.memset(GPA, val, 1)?;
            assert_eq!(unsafe { *hva }, val);
        }

        {
            // Test memset overflow
            memory_as.memset(GPA, 0, 512)?;
            assert!(memory_as.memset(GPA, 0, 513).is_err());
            assert!(memory_as.memset(GPA + 1, 0, 512).is_err());
        }

        {
            // Test copy_from_slice ok
            let val = 0xaa;
            memory_as.copy_from_slice(GPA, &[val; LEN], LEN)?;
            assert_eq!(unsafe { *hva }, val);
            assert_eq!(unsafe { *hva.add(LEN - 1) }, val);
        }

        {
            // Test copy_from_slice overflow
            assert!(
                memory_as
                    .copy_from_slice(GPA, &[0; LEN + 1], LEN + 1)
                    .is_err()
            );
            assert!(memory_as.copy_from_slice(GPA + 1, &[0; LEN], LEN).is_err());
        }

        Ok(())
    }
}
