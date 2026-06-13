use std::ops::Range;

use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;

const PORT: u16 = 0x70;
const LEN: usize = 2;

pub struct Cmos;

impl Cmos {
    pub fn new(pio_allocator: &mut RangeAllocator<u16>) -> Result<Self, DeviceError> {
        let _ = pio_allocator.reserve(PORT, LEN)?;

        Ok(Cmos)
    }
}

impl Device for Cmos {
    fn name(&self) -> String {
        "cmos".to_string()
    }

    fn support_pio_transport(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn support_pio_transport_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for Cmos {
    fn ports(&self) -> Vec<Range<u16>> {
        let range = PORT..PORT + LEN as u16;
        vec![range]
    }

    fn io_in(&self, _port: u16, _data: &mut [u8]) {
        // TODO
    }

    fn io_out(&self, _port: u16, _data: &[u8]) {
        // TODO
    }
}
