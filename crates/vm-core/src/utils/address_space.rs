use std::collections::BTreeMap;
use std::collections::btree_map::Iter;
use std::fmt::Debug;

#[derive(Debug, thiserror::Error)]
pub enum AddressSpaceError {
    #[error("invalid length of range")]
    InvalidLen,
    #[error("range overlap, offset: 0x{0:x}, len: {1}")]
    RangeOverlap(u64, usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Range<K>
where
    K: Debug + Into<u64>,
{
    pub start: K,
    pub len: usize,
}

pub struct AddressSpace<K, V>(BTreeMap<K, (usize, V)>)
where
    K: Debug;

impl<K, V> Default for AddressSpace<K, V>
where
    K: Debug,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, V> AddressSpace<K, V>
where
    K: Copy + Debug + Ord + Into<u64>,
{
    pub fn try_insert(&mut self, range: Range<K>, value: V) -> Result<(), AddressSpaceError> {
        if range.len == 0 {
            return Err(AddressSpaceError::InvalidLen);
        }

        if self.is_overlap(range.start, range.len) {
            return Err(AddressSpaceError::RangeOverlap(
                range.start.into(),
                range.len,
            ));
        }

        self.0.insert(range.start, (range.len, value));

        Ok(())
    }

    pub fn try_get_value_by_key(&self, key: K) -> Option<(Range<K>, &V)> {
        let (&start, &(len, ref value)) = self.0.range(..=key).next_back()?;

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

    pub fn iter(&self) -> Iter<'_, K, (usize, V)> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
