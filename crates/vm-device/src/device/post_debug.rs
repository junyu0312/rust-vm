use vm_core::device::Device;
use vm_core::device::PortRange;

const PORT: u16 = 0x80;

#[derive(Default)]
pub struct PostDebug;

impl Device for PostDebug {
    fn ports(&self) -> &[PortRange] {
        &[PortRange {
            start: PORT,
            len: 1,
        }]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        if port == PORT {}
    }
}
