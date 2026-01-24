use vm_core::device::Device;
use vm_core::device::PortRange;

pub struct Dummy;

impl Device for Dummy {
    fn ports(&self) -> &[PortRange] {
        // 0x40, 0x42, 0x43: PIT
        &[
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
