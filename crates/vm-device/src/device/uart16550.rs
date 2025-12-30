use crate::device::pio::PioDevice;

pub struct Uart16550 {}

impl Uart16550 {
    fn out_0x3fb(&self, data: &[u8]) {
        assert!(data.len() == 1);
        let byte = data[0];
        print!("{}", byte as char);
    }
}

impl PioDevice for Uart16550 {
    fn ports(&self) -> &[u16] {
        &[0x3FB]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        if port == 0x3FB {
            return self.out_0x3fb(data);
        }

        panic!("unsupported port {:#x} for uart16550", port);
    }
}
