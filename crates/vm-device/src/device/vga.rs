use crate::device::pio::PioDevice;

#[derive(Default)]
pub struct Vga;

impl PioDevice for Vga {
    fn ports(&self) -> &[u16] {
        &[0x3d4, 0x3d5]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            0x3d4 => {
                // Ignore
            }
            0x3d5 => {
                // Ignore
            }
            _ => todo!(),
        }
    }
}
