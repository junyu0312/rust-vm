use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

pub struct Dummy;

impl Device for Dummy {
    fn name(&self) -> String {
        "dummy".to_string()
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
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
