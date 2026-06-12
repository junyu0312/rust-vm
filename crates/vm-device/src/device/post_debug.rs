use std::ops::Range;

use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;

const PORT: u16 = 0x80;
const LEN: usize = 1;

pub struct PostDebug;

impl PostDebug {
    pub fn new(pio_allocator: &mut RangeAllocator<u16>) -> Result<Self, DeviceError> {
        let _ = pio_allocator.reserve(PORT, LEN);

        Ok(PostDebug)
    }
}

impl Device for PostDebug {
    fn name(&self) -> String {
        "post_debug".to_string()
    }

    fn support_pio_transport(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn support_pio_transport_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for PostDebug {
    fn ports(&self) -> Vec<Range<u16>> {
        vec![PORT..PORT + LEN as u16]
    }

    fn io_in(&self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&self, port: u16, _data: &[u8]) {
        if port == PORT {}
    }
}
