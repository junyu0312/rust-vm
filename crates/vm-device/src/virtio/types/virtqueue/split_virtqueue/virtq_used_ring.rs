pub const VIRTQ_USED_F_NO_NOTIFY: u16 = 1;

#[repr(C)]
pub struct VirtqUsedElem {
    /// Index of start of used descriptor chain
    pub id: u32,
    /// The number of bytes written into the device writable portion of
    /// the buffer described by the descriptor chain.
    pub len: u32,
}

pub struct VirtqUsed {
    queue_size: u16,
    buf: *mut u8,
}

impl VirtqUsed {
    fn addr_of_idx(&self) -> *const u16 {
        unsafe { (self.buf as *const u16).add(1) }
    }

    pub fn new(queue_size: u16, buf: *mut u8) -> Self {
        VirtqUsed { queue_size, buf }
    }

    pub fn flags(&self) -> u16 {
        unsafe { *(self.buf as *const u16) }
    }

    pub fn incr_idx(&mut self) {
        let mut val = self.idx();
        val += 1;
        val %= self.queue_size;
        unsafe { *(self.addr_of_idx() as *mut u16) = val };
    }

    pub fn idx(&self) -> u16 {
        unsafe { *self.addr_of_idx() }
    }

    /// Ring of used elements
    pub fn ring(&mut self, idx: u16) -> &mut VirtqUsedElem {
        unsafe {
            let base = self.buf.add(4);
            &mut *((base as *mut VirtqUsedElem).add(idx as usize))
        }
    }

    /// Only if VIRTIO_F_EVENT_IDX is negotiated
    pub fn avail_event(&self) -> u16 {
        assert_eq!(size_of::<VirtqUsedElem>(), 8);

        unsafe {
            *(self
                .buf
                .add(4 + size_of::<VirtqUsedElem>() * self.queue_size as usize)
                as *mut u16)
        }
    }
}
