use std::collections::BTreeMap;

use crate::error::Error;
use crate::memory_container::MemoryContainer;
use crate::region::MemoryRegion;

pub struct MemoryAddressSpace<C> {
    /// gpa |-> memory region
    regions: BTreeMap<u64, MemoryRegion<C>>,
}

impl<C> Default for MemoryAddressSpace<C> {
    fn default() -> Self {
        Self {
            regions: Default::default(),
        }
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

        let old = self.regions.insert(region.gpa, region);
        assert!(old.is_none());

        Ok(())
    }

    // TODO: the API is not friendly to multi regions, try to avoid expose hva?
    pub fn gpa_to_hva(&self, gpa: u64) -> Result<*mut u8, Error> {
        let region = self.try_get_region_by_gpa(gpa)?;
        let hva = region.hva();

        unsafe { Ok(hva.add((gpa - region.gpa) as usize)) }
    }

    pub fn memset(&self, mut gpa: u64, val: u8, len: usize) -> Result<(), Error> {
        let mut check_gpa = gpa;
        let mut remaining = len;

        while remaining > 0 {
            let region = self.try_get_region_by_gpa(check_gpa)?;

            let offset = check_gpa - region.gpa;
            let avail = region.len() - offset as usize;
            let step = remaining.min(avail);

            remaining -= step;
            check_gpa += step as u64;
        }

        remaining = len;

        while remaining > 0 {
            let region = self.try_get_region_by_gpa(gpa)?;

            let offset = gpa - region.gpa;
            let avail = region.len() - offset as usize;
            let step = remaining.min(avail);

            unsafe {
                region.hva().add(offset as usize).write_bytes(val, step);
            }

            remaining -= step;
            gpa += step as u64;
        }

        Ok(())
    }

    pub fn copy_from_slice(&self, mut gpa: u64, buf: &[u8]) -> Result<(), Error> {
        let mut remaining = buf.len();
        let mut check_gpa = gpa;

        while remaining > 0 {
            let region = self.try_get_region_by_gpa(check_gpa)?;

            let offset = check_gpa - region.gpa;
            let avail = region.len() - offset as usize;
            let step = remaining.min(avail);

            remaining -= step;
            check_gpa += step as u64;
        }

        remaining = buf.len();
        let mut src_offset = 0;

        while remaining > 0 {
            let region = self.try_get_region_by_gpa(gpa)?;

            let offset = gpa - region.gpa;
            let avail = region.len() - offset as usize;
            let step = remaining.min(avail);

            unsafe {
                region
                    .hva()
                    .add(offset as usize)
                    .copy_from_nonoverlapping(buf.as_ptr().add(src_offset), step);
            }

            remaining -= step;
            src_offset += step;
            gpa += step as u64;
        }

        Ok(())
    }

    fn is_overlapping(&self, region: &MemoryRegion<C>) -> bool {
        let new_left = region.gpa;
        let new_right = region.gpa + region.len() as u64;

        if let Some((_, prev)) = self.regions.range(..=new_left).next_back() {
            let prev_right = prev.gpa + prev.len() as u64;
            if prev_right > new_left {
                return true;
            }
        }

        if let Some((_, next)) = self.regions.range(new_left..).next() {
            let next_left = next.gpa;
            if next_left < new_right {
                return true;
            }
        }

        false
    }

    fn get_by_gpa(&self, gpa: u64) -> Option<&MemoryRegion<C>> {
        let (_, region) = self.regions.range(..=gpa).next_back()?;

        if gpa < region.gpa + region.len() as u64 {
            Some(region)
        } else {
            None
        }
    }

    fn try_get_region_by_gpa(&self, gpa: u64) -> Result<&MemoryRegion<C>, Error> {
        self.get_by_gpa(gpa).ok_or(Error::AccessInvalidGpa(gpa))
    }
}

#[cfg(test)]
mod tests {
    use memmap2::MmapMut;

    use crate::allocator::Allocator;
    use crate::allocator::mmap_allocator::MmapAllocator;
    use crate::manager::MemoryAddressSpace;
    use crate::region::MemoryRegion;

    #[test]
    fn test_memory_layout() -> anyhow::Result<()> {
        let mut memory_as = MemoryAddressSpace::<MmapMut>::default();

        let allocator = MmapAllocator;

        assert!(
            memory_as
                .try_insert(MemoryRegion::new(0, allocator.alloc(10, None)?))
                .is_ok()
        );
        assert!(
            memory_as
                .try_insert(MemoryRegion::new(5, allocator.alloc(10, None)?))
                .is_err()
        );
        assert!(
            memory_as
                .try_insert(MemoryRegion::new(10, allocator.alloc(10, None)?))
                .is_ok()
        );

        Ok(())
    }

    #[test]
    fn test_memory_write() -> anyhow::Result<()> {
        const GPA: u64 = 0;
        const LEN: usize = 512;

        let mut memory_as = MemoryAddressSpace::<MmapMut>::default();
        let allocator = MmapAllocator;

        let region = MemoryRegion::new(GPA, allocator.alloc(LEN, None)?);

        let hva = region.hva();

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
            memory_as.copy_from_slice(GPA, &[val; LEN])?;
            assert_eq!(unsafe { *hva }, val);
            assert_eq!(unsafe { *hva.add(LEN - 1) }, val);
        }

        {
            // Test copy_from_slice overflow
            assert!(memory_as.copy_from_slice(GPA, &[0; LEN + 1],).is_err());
            assert!(memory_as.copy_from_slice(GPA + 1, &[0; LEN],).is_err());
        }

        Ok(())
    }

    #[test]
    fn test_memset_multi_regions_ok() -> anyhow::Result<()> {
        let mut memory = MemoryAddressSpace::<MmapMut>::default();
        let allocator = MmapAllocator;

        let region0 = MemoryRegion::new(0, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region0).is_ok());

        let region1 = MemoryRegion::new(10, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region1).is_ok());

        assert!(memory.memset(0, 0xff, 20).is_ok());

        Ok(())
    }

    #[test]
    fn test_memset_multi_regions_fail() -> anyhow::Result<()> {
        let mut memory = MemoryAddressSpace::<MmapMut>::default();
        let allocator = MmapAllocator;

        let region0 = MemoryRegion::new(0, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region0).is_ok());

        let region1 = MemoryRegion::new(20, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region1).is_ok());

        assert!(memory.memset(0, 0xff, 20).is_err());

        Ok(())
    }

    #[test]
    fn test_copy_from_slice_multi_regions_ok() -> anyhow::Result<()> {
        let mut memory = MemoryAddressSpace::<MmapMut>::default();
        let allocator = MmapAllocator;

        let region0 = MemoryRegion::new(0, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region0).is_ok());

        let region1 = MemoryRegion::new(10, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region1).is_ok());

        assert!(memory.copy_from_slice(0, &[0xff; 20]).is_ok());

        Ok(())
    }

    #[test]
    fn test_copy_from_slice_multi_regions_fail() -> anyhow::Result<()> {
        let mut memory = MemoryAddressSpace::<MmapMut>::default();
        let allocator = MmapAllocator;

        let region0 = MemoryRegion::new(0, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region0).is_ok());

        let region1 = MemoryRegion::new(20, allocator.alloc(10, None)?);
        assert!(memory.try_insert(region1).is_ok());

        assert!(memory.copy_from_slice(0, &[0xff; 20]).is_err());

        Ok(())
    }
}
