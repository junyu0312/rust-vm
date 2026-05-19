use serde::Deserialize;
use serde::Serialize;

use crate::allocator::Allocator;
use crate::allocator::AllocatorKind;
use crate::allocator::mmap_allocator::MmapAllocator;
use crate::allocator::std_allocator::StdAllocator;
use crate::error::Error;
use crate::manager::MemoryAddressSpace;
use crate::region::MemoryRegion;
use crate::region::snapshot::MemoryRegionSnapshot;

#[derive(Serialize, Deserialize)]
pub struct MemoryAddressSpaceSnapshot {
    pub regions: Vec<MemoryRegionSnapshot>,
}

impl MemoryAddressSpace {
    pub fn build_snapshot(&self) -> Result<MemoryAddressSpaceSnapshot, Error> {
        let regions = self
            .regions
            .iter()
            .map(|(&gpa, region)| MemoryRegionSnapshot {
                gpa,
                align: region.align(),
                kind: region.kind(),
                buf: region.as_slice().to_vec(),
            })
            .collect();

        let snap = MemoryAddressSpaceSnapshot { regions };

        Ok(snap)
    }

    pub fn from_snapshot(snap: MemoryAddressSpaceSnapshot) -> Result<Self, Error> {
        let mut memory_address_space = MemoryAddressSpace::default();

        for region in snap.regions {
            let memory_region = match region.kind {
                AllocatorKind::Mmap => {
                    Box::new(MmapAllocator.alloc(region.buf.len(), region.align)?) as _
                }
                AllocatorKind::Std => {
                    Box::new(StdAllocator.alloc(region.buf.len(), region.align)?) as _
                }
            };

            let memory_region = MemoryRegion::new(region.gpa, memory_region);

            memory_address_space
                .try_insert(memory_region)
                .map_err(|_| Error::MemoryOverflow)?;
        }

        Ok(memory_address_space)
    }
}
