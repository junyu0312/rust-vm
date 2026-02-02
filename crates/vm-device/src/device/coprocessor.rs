use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

#[derive(Default)]
pub struct Coprocessor;

impl Device for Coprocessor {
    fn name(&self) -> String {
        "coprocessor".to_string()
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for Coprocessor {
    fn ports(&self) -> Vec<PortRange> {
        vec![
            PortRange {
                start: 0xf0,
                len: 1,
            },
            PortRange {
                start: 0xf1,
                len: 1,
            },
        ]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            0xf0 => {
                // ignore
            }
            0xf1 => {
                // ignore
            }
            _ => {}
        }
    }
}
