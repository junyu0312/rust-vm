use crate::device::pio::PioDevice;

#[derive(Default)]
pub struct Pic;

impl PioDevice for Pic {
    fn ports(&self) -> &[u16] {
        &[0xa1, 0x21]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            0xa1 => {
                // ignore
            }
            0x21 => {
                // ignore
            }
            _ => {}
        }
    }
}
