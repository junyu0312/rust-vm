use crate::device::pio::PioDevice;

const PORT: u16 = 0x80;

#[derive(Default)]
pub struct PostDebug;

impl PioDevice for PostDebug {
    fn ports(&self) -> &[u16] {
        &[PORT]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        if port == PORT {}
    }
}
