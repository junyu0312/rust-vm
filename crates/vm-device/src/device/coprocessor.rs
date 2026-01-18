use vm_core::device::pio::PioDevice;

#[derive(Default)]
pub struct Coprocessor;

impl PioDevice for Coprocessor {
    fn ports(&self) -> &[u16] {
        &[0xf0, 0xf1]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            0xf0 => {
                // ignore
            }
            0xf1 => {
                // ignore
            }
            _ => {}
        }
    }
}
