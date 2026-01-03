use crate::device::pio::PioDevice;

pub struct Serial;

impl PioDevice for Serial {
    fn ports(&self) -> &[u16] {
        &[0x3ff]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        for data in data {
            *data = 0xff;
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        // ignore
    }
}
