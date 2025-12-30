pub trait PioDevice {
    fn ports(&self) -> &[u16];
    fn io_in(&mut self, port: u16, data: &mut [u8]);
    fn io_out(&mut self, port: u16, data: &[u8]);
}
