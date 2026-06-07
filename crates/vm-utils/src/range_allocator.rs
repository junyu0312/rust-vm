use std::ops::Range;

use rangemap::RangeSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to insert")]
    InsertOverlap,

    #[error("Failed to alloc")]
    Alloc,

    #[error("Failed to reserve")]
    Reserve,

    #[error("Failed to free")]
    Free,
}

#[derive(Default)]
pub struct RangeAllocator<T> {
    free: RangeSet<T>,
    used: RangeSet<T>,
}

macro_rules! define_range_allocator {
    ($t:ty) => {
        impl RangeAllocator<$t> {
            #[inline]
            fn range(start: $t, len: usize) -> Range<$t> {
                start..start + len as $t
            }

            fn is_overlapping(lhs: &Range<$t>, rhs: &Range<$t>) -> bool {
                lhs.start < rhs.end && rhs.start < lhs.end
            }

            fn includes(lhs: &Range<$t>, rhs: &Range<$t>) -> bool {
                lhs.start <= rhs.start && rhs.end <= lhs.end
            }

            pub fn insert(&mut self, start: $t, len: usize) -> Result<(), Error> {
                let r = Self::range(start, len);

                if self.free.iter().any(|x| Self::is_overlapping(x, &r))
                    || self.used.iter().any(|x| Self::is_overlapping(x, &r))
                {
                    return Err(Error::InsertOverlap);
                }

                self.free.insert(r);

                Ok(())
            }

            pub fn alloc(&mut self, len: usize) -> Result<$t, Error> {
                let len = len as $t;

                let Some(range) = self.free.iter().find(|r| r.end - r.start >= len).cloned() else {
                    return Err(Error::Alloc);
                };

                let range = range.start..range.start + len as $t;

                self.free.remove(range.clone());
                self.used.insert(range.clone());

                Ok(range.start)
            }

            pub fn reserve(&mut self, start: $t, len: usize) -> Result<$t, Error> {
                let r = Self::range(start, len);

                let _ = self
                    .free
                    .iter()
                    .find(|x| Self::includes(x, &r))
                    .ok_or(Error::Reserve)?;

                self.free.remove(r.clone());
                self.used.insert(r.clone());

                Ok(start)
            }

            pub fn free(&mut self, start: $t, len: usize) -> Result<(), Error> {
                let r = Self::range(start, len);

                let _ = self
                    .used
                    .iter()
                    .find(|x| Self::includes(x, &r))
                    .ok_or(Error::Free)?;

                self.used.remove(r.clone());
                self.free.insert(r);

                Ok(())
            }
        }
    };
}

define_range_allocator!(u8);
define_range_allocator!(u16);
define_range_allocator!(u32);
define_range_allocator!(u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_allocator_insert_ok() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());
        assert!(allocator.insert(0x10000, 0x10000).is_ok());
    }

    #[test]
    fn test_range_allocator_insert_overlapping() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert!(allocator.insert(0, 0x10000).is_err());
        assert!(allocator.insert(0, 0x20000).is_err());
        assert!(allocator.insert(0x5000, 0x5000).is_err());
    }

    #[test]
    fn test_range_allocator_insert_used_overlapping() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert_eq!(allocator.alloc(0x1000).unwrap(), 0);
        assert!(allocator.insert(0, 0x1000).is_err());
    }

    #[test]
    fn test_range_allocator_insert_merging() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert!(allocator.insert(0x10000, 0x10000).is_ok());
        assert!(allocator.alloc(0x20000).is_ok());
    }

    #[test]
    fn test_range_allocator_alloc_continuous() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert_eq!(allocator.alloc(0x100).unwrap(), 0);
        assert_eq!(allocator.alloc(0x100).unwrap(), 0x100);
        assert_eq!(allocator.alloc(0x100).unwrap(), 0x200);
    }

    #[test]
    fn test_range_allocator_alloc_from_different_segments() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x100).is_ok());
        assert!(allocator.insert(0x10000, 0x100).is_ok());

        assert_eq!(allocator.alloc(0x50).unwrap(), 0);
        assert_eq!(allocator.alloc(0x100).unwrap(), 0x10000);
        assert_eq!(allocator.alloc(0x50).unwrap(), 0x50);
    }

    #[test]
    fn test_range_allocator_alloc_oom() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert!(allocator.alloc(0x20000).is_err());
    }

    #[test]
    fn test_range_allocator_reserve_ok() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert_eq!(allocator.reserve(0x100, 0x100).unwrap(), 0x100);
        assert_eq!(allocator.alloc(0x100).unwrap(), 0);
        assert_eq!(allocator.alloc(0x100).unwrap(), 0x200);
    }

    #[test]
    fn test_range_allocator_free_ok() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert_eq!(allocator.alloc(0x100).unwrap(), 0);
        assert!(allocator.free(0, 0x100).is_ok());
        assert_eq!(allocator.alloc(0x10000).unwrap(), 0);
    }

    #[test]
    fn test_range_allocator_free_merging() {
        let mut allocator = RangeAllocator::<u64>::default();

        assert!(allocator.insert(0, 0x10000).is_ok());

        assert_eq!(allocator.alloc(0x100).unwrap(), 0);
        assert_eq!(allocator.alloc(0x100).unwrap(), 0x100);
        assert!(allocator.free(0, 0x200).is_ok());
        assert_eq!(allocator.alloc(0x100).unwrap(), 0);
    }
}
