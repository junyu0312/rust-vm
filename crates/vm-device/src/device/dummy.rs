use vm_core::device::Device;
use vm_core::device::pio::pio_device::PioDevice;
use vm_core::device::pio::pio_device::PortRange;

pub struct Dummy;

impl Device for Dummy {
    fn name(&self) -> String {
        "dummy".to_string()
    }
}

impl PioDevice for Dummy {
    fn ports(&self) -> Vec<PortRange> {
        // 0x40, 0x42, 0x43: PIT
        vec![
            // PortRange {
            //     start: 0x40,
            //     len: 1,
            // },
            // PortRange {
            //     start: 0x42,
            //     len: 1,
            // },
            // PortRange {
            //     start: 0x43,
            //     len: 1,
            // },
            // PortRange {
            //     start: 0x87,
            //     len: 1,
            // },
        ]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {}

    fn io_out(&mut self, _port: u16, _data: &[u8]) {}
}
