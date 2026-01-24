use vm_core::device::Device;
use vm_core::device::PortRange;

#[derive(Default)]
pub struct Cmos;

impl Device for Cmos {
    fn ports(&self) -> &[PortRange] {
        &[
            PortRange {
                start: 0x70,
                len: 1,
            },
            PortRange {
                start: 0x71,
                len: 1,
            },
        ]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        // TODO
    }

    fn io_out(&mut self, _port: u16, _data: &[u8]) {
        // TODO
    }
}
