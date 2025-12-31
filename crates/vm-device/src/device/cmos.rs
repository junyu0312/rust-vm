use crate::device::pio::PioDevice;

#[derive(Default)]
pub struct Cmos;

impl PioDevice for Cmos {
    fn ports(&self) -> &[u16] {
        &[0x70]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        if port == 0x70 {
            // Handle CMOS register selection
        }
    }
}
