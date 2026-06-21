use std::io::Read;
use std::io::Write;

use vm_core::device::error::DeviceSnapshotError;
use vm_mm::manager::MemoryAddressSpace;
use vm_snapshot::helper::read_u8;
use vm_snapshot::helper::read_u16;
use vm_snapshot::helper::read_u32;
use vm_snapshot::helper::write_bool;
use vm_snapshot::helper::write_u16;
use vm_snapshot::helper::write_u32;

use crate::result::VirtioError;
use crate::virtqueue::virtq_avail_ring::VirtqAvail;
use crate::virtqueue::virtq_desc_table::VirtqDescTableRef;
use crate::virtqueue::virtq_used_ring::VirtqUsed;

pub mod virtq_avail_ring;
pub mod virtq_desc_table;
pub mod virtq_used_ring;

fn to_gpa(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) + (low as u64)
}

#[derive(Clone, Copy)]
pub struct Virtqueue {
    queue_size_max: u16,
    queue_size: u16,
    queue_ready: bool,
    queue_desc_low: u32,
    queue_desc_high: u32,
    queue_available_low: u32,
    queue_available_high: u32,
    queue_used_low: u32,
    queue_used_high: u32,
}

impl Virtqueue {
    pub fn new(queue_size_max: u16) -> Self {
        Virtqueue {
            queue_size_max,
            queue_size: queue_size_max, // virtio-pci uses it as maximal queue size, does it work on mmio?
            queue_ready: Default::default(),
            queue_desc_low: Default::default(),
            queue_desc_high: Default::default(),
            queue_available_low: Default::default(),
            queue_available_high: Default::default(),
            queue_used_low: Default::default(),
            queue_used_high: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.queue_size = self.queue_size_max; // virtio-pci uses it as maximal queue size, does it work on mmio?
        self.queue_ready = Default::default();
        self.queue_desc_low = Default::default();
        self.queue_desc_high = Default::default();
        self.queue_available_low = Default::default();
        self.queue_available_high = Default::default();
        self.queue_used_low = Default::default();
        self.queue_used_high = Default::default();
    }

    pub fn read_queue_size_max(&self) -> u16 {
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
        self.queue_desc_low = addr;
    }

    pub fn write_queue_desc_high(&mut self, addr: u32) {
        self.queue_desc_high = addr;
    }

    pub fn write_queue_available_low(&mut self, addr: u32) {
        self.queue_available_low = addr;
    }

    pub fn write_queue_available_high(&mut self, addr: u32) {
        self.queue_available_high = addr;
    }

    pub fn write_queue_used_low(&mut self, addr: u32) {
        self.queue_used_low = addr;
    }

    pub fn write_queue_used_high(&mut self, addr: u32) {
        self.queue_used_high = addr;
    }

    pub fn desc_table_ref(
        &self,
        mm: &MemoryAddressSpace,
    ) -> Result<VirtqDescTableRef, VirtioError> {
        let gpa = self.queue_desc_table_gpa();
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtioError::AccessInvalidGpa(gpa))?;

        Ok(VirtqDescTableRef::new(self.queue_size, hva))
    }

    pub fn avail_ring(&self, mm: &MemoryAddressSpace) -> Result<VirtqAvail, VirtioError> {
        let gpa = self.queue_available_ring_gpa();
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtioError::AccessInvalidGpa(gpa))?;

        Ok(VirtqAvail::new(self.queue_size, hva as *const u16))
    }

    pub fn used_ring(&self, mm: &MemoryAddressSpace) -> Result<VirtqUsed, VirtioError> {
        let gpa = self.queue_used_ring_gpa();
        let hva = mm
            .gpa_to_hva(gpa)
            .map_err(|_| VirtioError::AccessInvalidGpa(gpa))?;

        Ok(VirtqUsed::new(self.queue_size, hva))
    }

    fn queue_desc_table_gpa(&self) -> u64 {
        to_gpa(self.queue_desc_high, self.queue_desc_low)
    }

    fn queue_available_ring_gpa(&self) -> u64 {
        to_gpa(self.queue_available_high, self.queue_available_low)
    }

    fn queue_used_ring_gpa(&self) -> u64 {
        to_gpa(self.queue_used_high, self.queue_used_low)
    }

    pub fn save(&self, writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        write_u16(writer, self.queue_size)?;
        write_bool(writer, self.queue_ready)?;
        write_u32(writer, self.queue_desc_low)?;
        write_u32(writer, self.queue_desc_high)?;
        write_u32(writer, self.queue_available_low)?;
        write_u32(writer, self.queue_available_high)?;
        write_u32(writer, self.queue_used_low)?;
        write_u32(writer, self.queue_used_high)?;
        // write_u16(writer, self.last_available_idx)?;

        Ok(())
    }

    pub fn load(&mut self, reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        self.queue_size = read_u16(reader)?;
        self.queue_ready = read_u8(reader)? == 1;
        self.queue_desc_low = read_u32(reader)?;
        self.queue_desc_high = read_u32(reader)?;
        self.queue_available_low = read_u32(reader)?;
        self.queue_available_high = read_u32(reader)?;
        self.queue_used_low = read_u32(reader)?;
        self.queue_used_high = read_u32(reader)?;
        // self.last_available_idx = read_u16(reader)?;

        Ok(())
    }
}
