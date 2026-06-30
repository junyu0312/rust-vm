use std::sync::Mutex;

use thiserror::Error;
use vm_utils::range_allocator::RangeAllocator;

#[derive(Error, Debug)]
pub enum InterruptManagerError {
    #[error("Invalid argument")]
    InvalidArgument,

    #[error("Failed to reserve irq {0}")]
    ReserveIrq(u32),

    #[error("Failed to allocate irq")]
    AllocateIrq,

    #[error("Failed to allocate gsi")]
    AllocateGsi,
}

struct Allocator(Mutex<RangeAllocator<u32>>);

impl Allocator {
    fn new(start: u32, len: usize) -> Result<Self, InterruptManagerError> {
        let mut irq_allocator = RangeAllocator::<u32>::default();
        irq_allocator
            .insert(start, len)
            .map_err(|_| InterruptManagerError::InvalidArgument)?;
        Ok(Allocator(Mutex::new(irq_allocator)))
    }
}

pub struct InterruptManager {
    // We can use bitmap for better performance.
    irq_allocator: Allocator,
    #[cfg(target_arch = "x86_64")]
    gsi_allocator: Allocator,
}

impl InterruptManager {
    pub fn new(
        irq_start: u32,
        irq_len: usize,
        #[cfg(target_arch = "x86_64")] gsi_start: u32,
        #[cfg(target_arch = "x86_64")] gsi_len: usize,
    ) -> Result<Self, InterruptManagerError> {
        let irq_allocator = Allocator::new(irq_start, irq_len)?;

        #[cfg(target_arch = "x86_64")]
        let gsi_allocator = Allocator::new(gsi_start, gsi_len)?;

        Ok(InterruptManager {
            irq_allocator,
            #[cfg(target_arch = "x86_64")]
            gsi_allocator,
        })
    }

    pub fn reserve_irq(&self, irq: u32) -> Result<(), InterruptManagerError> {
        let mut irq_allocator = self.irq_allocator.0.lock().unwrap();
        let _ = irq_allocator
            .reserve(irq, 1)
            .map_err(|_| InterruptManagerError::ReserveIrq(irq))?;

        Ok(())
    }

    pub fn allocate_irq(&self) -> Result<u32, InterruptManagerError> {
        let mut irq_allocator = self.irq_allocator.0.lock().unwrap();
        let range = irq_allocator
            .alloc(1)
            .map_err(|_| InterruptManagerError::AllocateIrq)?;

        Ok(range.start)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn allocate_gsi(&self) -> Result<u32, InterruptManagerError> {
        let mut gsi_allocator = self.gsi_allocator.0.lock().unwrap();
        let range = gsi_allocator
            .alloc(1)
            .map_err(|_| InterruptManagerError::AllocateGsi)?;

        Ok(range.start)
    }

    #[cfg(target_arch = "aarch64")]
    pub fn allocate_gsi(&self) -> Result<u32, InterruptManagerError> {
        self.allocate_irq()
            .map_err(|_| InterruptManagerError::AllocateGsi)
    }
}
