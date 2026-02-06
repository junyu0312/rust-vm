use vm_core::device::Device;
use vm_core::device::pio::pio_device::PioDevice;
use vm_core::device::pio::pio_device::PortRange;

#[derive(Default)]
pub struct Vga;

impl Device for Vga {
    fn name(&self) -> String {
        "vga".to_string()
    }
}

impl PioDevice for Vga {
    fn ports(&self) -> Vec<PortRange> {
        vec![
            PortRange {
                start: 0x3d4,
                len: 1,
            },
            PortRange {
                start: 0x3d5,
                len: 1,
            },
        ]
    }

    fn io_in(&mut self, _port: u16, _data: &mut [u8]) {
        todo!()
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            0x3d4 => {
                // Ignore
            }
            0x3d5 => {
                // Ignore
            }
            _ => todo!(),
        }
    }
}
