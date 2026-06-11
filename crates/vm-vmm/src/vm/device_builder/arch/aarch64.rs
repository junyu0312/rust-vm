use vm_core::arch::aarch64::layout::*;

use vm_device::device::pl011::Pl011;
use vm_utils::range_allocator::RangeAllocator;

use crate::device::error::InitDeviceError;
use crate::vm::device_builder::DeviceManagerBuilder;

pub fn mmio_allocator() -> RangeAllocator<u64> {
    let mut allocator = RangeAllocator::<u64>::default();

    allocator.insert(MMIO_START as u64, MMIO_LEN as usize);
    allocator.insert(
        PCI_BAR_MMIO_WINDOW_START as u64,
        PCI_BAR_MMIO_WINDOW_LENGTH as usize,
    );
    allocator.insert(ECAM_BASE as u64, ECAM_LENGTH as usize);
    allocator
}

impl<'a> DeviceManagerBuilder<'a> {
    pub fn init_device_arch(&mut self) -> Result<(), InitDeviceError> {
        {
            let pl011 = Pl011::new(self.irq_allocator.alloc(), self.irq_chip.clone());
            self.device_manager.attach_device(Box::new(pl011))?;
        }

        Ok(())
    }
}
