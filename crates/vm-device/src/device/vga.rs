use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

#[derive(Default)]
pub struct Vga;

impl Device for Vga {
    fn name(&self) -> String {
        "vga".to_string()
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
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
