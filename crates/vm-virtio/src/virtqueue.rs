use vm_mm::allocator::MemoryContainer;
use vm_mm::manager::MemoryAddressSpace;

use crate::result::Result;
use crate::result::VirtioError;
use crate::virtqueue::virtq_avail_ring::VirtqAvail;
use crate::virtqueue::virtq_desc_table::VirtqDescTableRef;
use crate::virtqueue::virtq_used_ring::VirtqUsed;

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

pub struct Virtqueue {
    queue_size_max: u32,
    queue_size: u16,
    queue_ready: bool,
    queue_desc_low: Option<u32>,
    queue_desc_high: Option<u32>,
    queue_available_low: Option<u32>,
    queue_available_high: Option<u32>,
    queue_used_low: Option<u32>,
    queue_used_high: Option<u32>,
    last_available_idx: u16,
}

impl Virtqueue {
    pub fn new(queue_size_max: u32) -> Self {
        Virtqueue {
            queue_size_max,
            queue_size: queue_size_max.try_into().unwrap(), // virtio-pci uses it as maximal queue size, does it work on mmio?
            queue_ready: Default::default(),
            queue_desc_low: Default::default(),
            queue_desc_high: Default::default(),
            queue_available_low: Default::default(),
            queue_available_high: Default::default(),
            queue_used_low: Default::default(),
            queue_used_high: Default::default(),
            last_available_idx: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.queue_size = self.queue_size_max.try_into().unwrap(); // virtio-pci uses it as maximal queue size, does it work on mmio?
        self.queue_ready = Default::default();
        self.queue_desc_low = Default::default();
        self.queue_desc_high = Default::default();
        self.queue_available_low = Default::default();
        self.queue_available_high = Default::default();
        self.queue_used_low = Default::default();
        self.queue_used_high = Default::default();
        self.last_available_idx = Default::default();
    }

    pub fn read_queue_size_max(&self) -> u32 {
        self.queue_size_max
    }

    pub fn read_queue_size(&self) -> u16 {
        self.queue_size
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
        self.queue_desc_low = Some(addr);
    }

    pub fn write_queue_desc_high(&mut self, addr: u32) {
        self.queue_desc_high = Some(addr);
    }

    pub fn write_queue_available_low(&mut self, addr: u32) {
        self.queue_available_low = Some(addr);
    }

    pub fn write_queue_available_high(&mut self, addr: u32) {
        self.queue_available_high = Some(addr);
    }

    pub fn write_queue_used_low(&mut self, addr: u32) {
        self.queue_used_low = Some(addr);
    }

    pub fn write_queue_used_high(&mut self, addr: u32) {
        self.queue_used_high = Some(addr);
    }

    pub fn desc_table_ref<C>(&self, mm: &MemoryAddressSpace<C>) -> Result<VirtqDescTableRef>
    where
        C: MemoryContainer,
    {
        let gpa = self
            .queue_desc_table_gpa()
            .ok_or(VirtioError::AccessVirtqueueNotReady)?;
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtioError::AccessInvalidGpa(gpa))?;

        Ok(VirtqDescTableRef::new(self.queue_size, hva))
    }

    pub fn avail_ring<C>(&self, mm: &MemoryAddressSpace<C>) -> Result<VirtqAvail>
    where
        C: MemoryContainer,
    {
        let gpa = self
            .queue_available_ring_gpa()
            .ok_or(VirtioError::AccessVirtqueueNotReady)?;
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtioError::AccessInvalidGpa(gpa))?;

        Ok(VirtqAvail::new(self.queue_size, hva as *const u16))
    }

    pub fn used_ring<C>(&self, mm: &MemoryAddressSpace<C>) -> Result<VirtqUsed>
    where
        C: MemoryContainer,
    {
        let gpa = self
            .queue_used_ring_gpa()
            .ok_or(VirtioError::AccessVirtqueueNotReady)?;
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtioError::AccessInvalidGpa(gpa))?;

        Ok(VirtqUsed::new(self.queue_size, hva))
    }

    pub fn last_available_idx(&self) -> u16 {
        self.last_available_idx
    }

    pub fn incr_last_available_idx(&mut self) {
        self.last_available_idx = (self.last_available_idx + 1) % self.queue_size;
    }

    fn queue_desc_table_gpa(&self) -> Option<u64> {
        to_gpa(self.queue_desc_high.as_ref(), self.queue_desc_low.as_ref())
    }

    fn queue_available_ring_gpa(&self) -> Option<u64> {
        to_gpa(
            self.queue_available_high.as_ref(),
            self.queue_available_low.as_ref(),
        )
    }

    fn queue_used_ring_gpa(&self) -> Option<u64> {
        to_gpa(self.queue_used_high.as_ref(), self.queue_used_low.as_ref())
    }
}
