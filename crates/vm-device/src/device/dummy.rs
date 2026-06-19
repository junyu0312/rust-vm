use std::ops::Range;

use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;

pub struct Dummy;

impl Dummy {
    pub fn new(pio_allocator: &mut RangeAllocator<u16>) -> Result<Self, DeviceError> {
        let _ = pio_allocator.reserve(0x87, 1)?;

        Ok(Dummy)
    }
}

impl Device for Dummy {
    fn name(&self) -> String {
        "dummy".to_string()
    }

    fn support_pio_transport(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn support_pio_transport_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for Dummy {
    fn ports(&self) -> Vec<Range<u16>> {
        vec![
            // TODO
            0x87..0x88,
        ]
    }

    fn io_in(&self, _port: u16, _data: &mut [u8]) -> Result<(), DeviceError> {
        Ok(())
    }

    fn io_out(&self, _port: u16, _data: &[u8]) -> Result<(), DeviceError> {
        Ok(())
    }
}
