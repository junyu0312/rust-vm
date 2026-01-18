use vm_core::device::pio::PioDevice;

#[derive(Default)]
pub struct Cmos;

impl PioDevice for Cmos {
    fn ports(&self) -> &[u16] {
        &[0x70, 0x71]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        // TODO
    }

    fn io_out(&mut self, _port: u16, _data: &[u8]) {
        // TODO
    }
}
