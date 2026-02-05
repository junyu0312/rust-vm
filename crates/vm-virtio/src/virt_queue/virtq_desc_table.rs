/* This marks a buffer as continuing via the next field. */
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
/* This marks a buffer as device write-only (otherwise device read-only). */
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
/* This means the buffer contains a list of buffer descriptors. */
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

#[derive(Debug)]
#[repr(C, packed)]
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
    queue_size: u16,
    table: *mut VirtqDesc,
}

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
}
