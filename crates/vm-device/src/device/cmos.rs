use vm_core::device::Device;
use vm_core::device::pio::PioDevice;
use vm_core::device::pio::PortRange;

#[derive(Default)]
pub struct Cmos;

impl Device for Cmos {
    fn name(&self) -> String {
        "cmos".to_string()
    }

    fn as_pio_device(&self) -> Option<&dyn PioDevice> {
        Some(self)
    }

    fn as_pio_device_mut(&mut self) -> Option<&mut dyn PioDevice> {
        Some(self)
    }
}

impl PioDevice for Cmos {
    fn ports(&self) -> Vec<PortRange> {
        vec![
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
