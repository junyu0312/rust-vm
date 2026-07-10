use std::ptr::NonNull;
use std::slice;

use vm_mm::manager::MemoryAddressSpace;
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;
use zerocopy::KnownLayout;

use crate::result::Result;
use crate::result::VirtioError;

/// This marks a buffer as continuing via the next field.
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
/// This marks a buffer as device write-only (otherwise device read-only).
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
/// This means the buffer contains a list of buffer descriptors.
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

#[derive(Debug)]
#[repr(C, packed)]
pub struct VirtqDesc {
    /// Address (guest-physical).
    addr: u64,
    /// Length.
    pub len: u32,
    /// The flags as indicated above.
    pub flags: u16,
    /// Next field if flags & NEXT
    pub next: u16,
}

impl VirtqDesc {
    /// Get hva of the buf
    pub fn addr(&self, mm: &MemoryAddressSpace) -> Result<NonNull<u8>> {
        let addr = mm
            .gpa_to_hva(self.addr)
            .map_err(|_| VirtioError::AccessInvalidGpa(self.addr))?;
        NonNull::new(addr).ok_or(VirtioError::AccessInvalidGpa(self.addr))
    }

    pub fn as_ref<T>(&self, memory: &MemoryAddressSpace) -> Result<&T>
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        let req: NonNull<u8> = self.addr(memory)?;

        let bytes = unsafe { slice::from_raw_parts(req.as_ptr(), size_of::<T>()) };
        let t = T::ref_from_bytes(bytes).map_err(|_| VirtioError::TransmuteDesc)?;

        Ok(t)
    }

    // TODO: Refine virtqueue API
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut<T>(&self, memory: &MemoryAddressSpace) -> Result<&mut T>
    where
        T: FromBytes + IntoBytes + KnownLayout,
    {
        let req: NonNull<u8> = self.addr(memory)?;

        let bytes = unsafe { slice::from_raw_parts_mut(req.as_ptr(), size_of::<T>()) };
        let t = T::mut_from_bytes(bytes).map_err(|_| VirtioError::TransmuteDesc)?;

        Ok(t)
    }

    pub fn as_slice<T>(&self, memory: &MemoryAddressSpace, len: usize) -> Result<Vec<&T>>
    where
        T: FromBytes + KnownLayout + Immutable,
    {
        let req = self.addr(memory)?;
        let elem_size = size_of::<T>();

        let total_size = len
            .checked_mul(elem_size)
            .ok_or(VirtioError::TransmuteDesc)?;

        if total_size > self.len as usize {
            return Err(VirtioError::TransmuteDesc);
        }

        let bytes = unsafe { slice::from_raw_parts(req.as_ptr(), total_size) };

        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let start = i * elem_size;
            let end = start + elem_size;
            let item =
                T::ref_from_bytes(&bytes[start..end]).map_err(|_| VirtioError::TransmuteDesc)?;
            result.push(item);
        }

        Ok(result)
    }
}

pub struct VirtqDescTableRef {
    queue_size: u16,
    table: *mut VirtqDesc,
}
unsafe impl Send for VirtqDescTableRef {}
unsafe impl Sync for VirtqDescTableRef {}

impl VirtqDescTableRef {
    pub fn new(queue_size: u16, table: *mut u8) -> Self {
        VirtqDescTableRef {
            queue_size,
            table: table as *mut VirtqDesc,
        }
    }

    pub fn queue_size(&self) -> u16 {
        self.queue_size
    }

    pub fn get(&self, idx: u16) -> &VirtqDesc {
        unsafe { &*self.table.add(idx as usize) }
    }

    pub fn get_mut(&mut self, idx: u16) -> &mut VirtqDesc {
        unsafe { &mut *self.table.add(idx as usize) }
    }

    pub fn get_chain(&self, first_idx: u16) -> Vec<&VirtqDesc> {
        let mut descs = vec![];

        let mut curr = self.get(first_idx);
        descs.push(curr);
        while curr.flags & VIRTQ_DESC_F_NEXT != 0 {
            curr = self.get(curr.next);
            descs.push(curr);
        }

        descs
    }

    pub fn get_chain_mut(&self, first_idx: u16) -> Vec<&VirtqDesc> {
        let mut descs = vec![];

        let mut curr = self.get(first_idx);
        descs.push(curr);
        while curr.flags & VIRTQ_DESC_F_NEXT != 0 {
            curr = self.get(curr.next);
            descs.push(curr);
        }

        descs
    }
}
