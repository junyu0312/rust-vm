/* This marks a buffer as continuing via the next field. */
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
/* This marks a buffer as device write-only (otherwise device read-only). */
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
/* This means the buffer contains a list of buffer descriptors. */
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

#[repr(C)]
pub struct VirtqDesc {
    /// Address (guest-physical).
    pub addr: u64,
    /// Length.
    pub len: u32,
    /// The flags as indicated above.
    pub flags: u16,
    /// Next field if flags & NEXT
    pub next: u16,
}

pub struct VirtqDescTableRef {
    queue_size: usize,
    table: *mut VirtqDesc,
}

impl VirtqDescTableRef {
    pub fn new(queue_size: u16, table: *mut u8) -> Self {
        VirtqDescTableRef {
            queue_size: queue_size as usize,
            table: table as *mut VirtqDesc,
        }
    }

    pub fn get_mut(&mut self, idx: u16) -> &mut VirtqDesc {
        unsafe { &mut *self.table.add(idx as usize) }
    }
}
