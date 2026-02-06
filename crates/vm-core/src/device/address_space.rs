use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::device::Error;
use crate::device::Result;

#[derive(Clone, Copy, Debug)]
pub struct Range<K: Debug> {
    pub start: K,
    pub len: usize,
}

#[derive(Default)]
pub struct AddressSpace<K: Debug>(BTreeMap<K, (usize, usize)>); // start |-> (len, device)

impl<K> AddressSpace<K>
where
    K: Copy + Debug + Ord + Into<u64>,
{
    pub fn new() -> Self {
        AddressSpace(BTreeMap::new())
    }

    pub fn try_insert(&mut self, range: Range<K>, device: usize) -> Result<()> {
        if range.len == 0 {
            return Err(Error::InvalidLen);
        }

        if self.is_overlap(range.start, range.len) {
            return Err(Error::InvalidRange);
        }

        self.0.insert(range.start, (range.len, device));

        Ok(())
    }

    pub fn try_get_value_by_key(&self, key: K) -> Option<(Range<K>, usize)> {
        let (&start, &(len, value)) = self.0.range(..=key).next_back()?;

        if key.into() - start.into() < len as u64 {
            Some((Range { start, len }, value))
        } else {
            None
        }
    }

    pub fn is_overlap(&self, start: K, len: usize) -> bool {
        let end = start.into() + len as u64;

        if let Some((&left_start, &(left_len, _))) = self.0.range(..=start).next_back() {
            let left_start = left_start.into();
            let left_end = left_start + left_len as u64;

            if left_end > start.into() {
                return true;
            }
        }

        if let Some((&right_start, &(_, _))) = self.0.range(start..).next() {
            let right_start = right_start.into();

            if end > right_start {
                return true;
            }
        }

        false
    }
}
