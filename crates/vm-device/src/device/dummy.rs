use vm_core::device::pio::PioDevice;

pub struct Dummy;

impl PioDevice for Dummy {
    fn ports(&self) -> &[u16] {
        // 0x40, 0x42, 0x43: PIT
        // 0xc00a, 0xc000: I dont know
        &[
            0x40, 0x42, 0x43, 0xc00a, 0xc000, 0xc10a, 0xc100, 0xc20a, 0xc200, 0xc30a, 0xc300,
            0xc40a, 0xc400, 0xc50a, 0x87,
        ]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {}

    fn io_out(&mut self, _port: u16, _data: &[u8]) {}
}
