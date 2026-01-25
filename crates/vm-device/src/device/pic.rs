use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

#[derive(Default)]
pub struct Pic;

impl Device for Pic {
    fn name(&self) -> &str {
        "pic"
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for Pic {
    fn ports(&self) -> Vec<PortRange> {
        vec![
            PortRange {
                start: 0xa1,
                len: 1,
            },
            PortRange {
                start: 0x21,
                len: 1,
            },
        ]
    }

    fn io_in(&mut self, port: u16, _data: &mut [u8]) {
        match port {
            0xa1 => (),
            0x21 => (),
            _ => {}
        }
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            0xa1 => {
                // ignore
            }
            0x21 => {
                // ignore
            }
            _ => {}
        }
    }
}
