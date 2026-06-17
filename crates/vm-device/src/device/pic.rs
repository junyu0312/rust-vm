use std::ops::Range;

use vm_core::device::Device;
use vm_core::device::error::DeviceError;
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;

const PORT0: u16 = 0xa1;
const PORT0_LEN: usize = 1;
const PORT1: u16 = 0x21;
const PORT1_LEN: usize = 1;

pub struct Pic;

impl Pic {
    pub fn new(pio_allocator: &mut RangeAllocator<u16>) -> Result<Self, DeviceError> {
        let _ = pio_allocator.reserve(PORT0, PORT0_LEN)?;
        let _ = pio_allocator.reserve(PORT1, PORT1_LEN)?;

        Ok(Pic)
    }
}

impl Device for Pic {
    fn name(&self) -> String {
        "pic".to_string()
    }
}

impl PioDevice for Pic {
    fn ports(&self) -> Vec<Range<u16>> {
        vec![
            PORT0..PORT0 + PORT0_LEN as u16,
            PORT1..PORT1 + PORT1_LEN as u16,
        ]
    }

    fn io_in(&self, port: u16, _data: &mut [u8]) -> Result<(), DeviceError> {
        match port {
            0xa1 => (),
            0x21 => (),
            _ => {}
        }

        Ok(())
    }

    fn io_out(&self, port: u16, _data: &[u8]) -> Result<(), DeviceError> {
        match port {
            0xa1 => {
                // ignore
            }
            0x21 => {
                // ignore
            }
            _ => {}
        }

        Ok(())
    }
}
