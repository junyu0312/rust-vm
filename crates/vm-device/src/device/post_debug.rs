use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

const PORT: u16 = 0x80;

#[derive(Default)]
pub struct PostDebug;

impl Device for PostDebug {
    fn name(&self) -> &str {
        "post_debug"
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for PostDebug {
    fn ports(&self) -> Vec<PortRange> {
        vec![PortRange {
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
