use std::cell::OnceCell;

use tracing::warn;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;

use crate::result::Result;
use crate::result::VirtIoError;
use crate::virt_queue::virtq_avail_ring::VirtqAvail;
use crate::virt_queue::virtq_desc_table::VirtqDescTableRef;
use crate::virt_queue::virtq_used_ring::VirtqUsed;

pub mod virtq_avail_ring;
pub mod virtq_desc_table;
pub mod virtq_used_ring;

fn to_gpa(high: Option<&u32>, low: Option<&u32>) -> Option<u64> {
    if let Some(&low) = low
        && let Some(&high) = high
    {
        Some(((high as u64) << 32) + (low as u64))
    } else {
        None
    }
}

pub struct VirtQueue {
    queue_size_max: u32,
    queue_size: u16,
    queue_ready: bool,
    queue_desc_low: OnceCell<u32>,
    queue_desc_high: OnceCell<u32>,
    queue_available_low: OnceCell<u32>,
    queue_available_high: OnceCell<u32>,
    queue_used_low: OnceCell<u32>,
    queue_used_high: OnceCell<u32>,
}

impl VirtQueue {
    pub fn new(queue_size_max: u32) -> Self {
        VirtQueue {
            queue_size_max,
            queue_size: Default::default(),
            queue_ready: Default::default(),
            queue_desc_low: Default::default(),
            queue_desc_high: Default::default(),
            queue_available_low: Default::default(),
            queue_available_high: Default::default(),
            queue_used_low: Default::default(),
            queue_used_high: Default::default(),
        }
    }

    pub fn read_queue_size_max(&self) -> u32 {
        self.queue_size_max
    }

    pub fn write_queue_size(&mut self, queue_size: u16) {
        self.queue_size = queue_size;
    }

    pub fn read_queue_ready(&self) -> bool {
        self.queue_ready
    }

    pub fn write_queue_ready(&mut self, queue_ready: bool) {
        self.queue_ready = queue_ready;
    }

    pub fn write_queue_desc_low(&mut self, addr: u32) {
        if self.queue_desc_low.set(addr).is_err() {
            warn!("repeated writes to queue_desc_low are ignored")
        }
    }

    pub fn write_queue_desc_high(&mut self, addr: u32) {
        if self.queue_desc_high.set(addr).is_err() {
            warn!("repeated writes to queue_desc_high are ignored")
        }
    }

    pub fn write_queue_available_low(&mut self, addr: u32) {
        if self.queue_available_low.set(addr).is_err() {
            warn!("repeated writes to queue_available_low are ignored")
        }
    }

    pub fn write_queue_available_high(&mut self, addr: u32) {
        if self.queue_available_high.set(addr).is_err() {
            warn!("repeated writes to queue_available_high are ignored")
        }
    }

    pub fn write_queue_used_low(&mut self, addr: u32) {
        if self.queue_used_low.set(addr).is_err() {
            warn!("repeated writes to queue_used_low are ignored")
        }
    }

    pub fn write_queue_used_high(&mut self, addr: u32) {
        if self.queue_used_high.set(addr).is_err() {
            warn!("repeated writes to queue_used_high are ignored")
        }
    }

    pub fn desc_table_ref<C>(&self, mm: &mut MemoryAddressSpace<C>) -> Result<VirtqDescTableRef>
    where
        C: MemoryContainer,
    {
        let gpa = self
            .queue_desc_table_gpa()
            .ok_or(VirtIoError::AccessVirtqueueNotReady)?;
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtIoError::AccessInvalidGpa)?;

        Ok(VirtqDescTableRef::new(self.queue_size, hva))
    }

    pub fn avail_ring<C>(&self, mm: &mut MemoryAddressSpace<C>) -> Result<VirtqAvail>
    where
        C: MemoryContainer,
    {
        let gpa = self
            .queue_available_ring_gpa()
            .ok_or(VirtIoError::AccessVirtqueueNotReady)?;
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtIoError::AccessInvalidGpa)?;

        Ok(VirtqAvail::new(self.queue_size, hva as *const u16))
    }

    pub fn used_ring<C>(&self, mm: &mut MemoryAddressSpace<C>) -> Result<VirtqUsed>
    where
        C: MemoryContainer,
    {
        let gpa = self
            .queue_used_ring_gpa()
            .ok_or(VirtIoError::AccessVirtqueueNotReady)?;
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtIoError::AccessInvalidGpa)?;

        Ok(VirtqUsed::new(self.queue_size, hva))
    }

    fn queue_desc_table_gpa(&self) -> Option<u64> {
        to_gpa(self.queue_desc_high.get(), self.queue_desc_low.get())
    }

    fn queue_available_ring_gpa(&self) -> Option<u64> {
        to_gpa(
            self.queue_available_high.get(),
            self.queue_available_low.get(),
        )
    }

    fn queue_used_ring_gpa(&self) -> Option<u64> {
        to_gpa(self.queue_used_high.get(), self.queue_used_low.get())
    }
}
