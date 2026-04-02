use std::sync::Mutex;

use vm_core::device::Device;
use vm_core::device::pio::pio_device::PioDevice;
use vm_core::device::pio::pio_device::PortRange;

use crate::root_complex::PciRootComplex;
use crate::root_complex::pio::config_addr::ConfigAddress;

mod config_addr;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_DATA: u16 = 0xcfc;

#[derive(Default)]
pub struct PciRootComplexPio {
    config_address: Mutex<ConfigAddress>,
    internal: Mutex<PciRootComplex>,
}

impl PciRootComplexPio {
    fn handle_out_config_address(&self, offset: u8, data: &[u8]) {
        self.config_address.lock().unwrap().write(offset, data);
    }

    fn handle_in_config_address(&self, offset: u8, data: &mut [u8]) {
        self.config_address.lock().unwrap().read(offset, data);
    }

    fn handle_out_config_data(&self, offset: u8, data: &[u8]) {
        let config_address = self.config_address.lock().unwrap();

        assert_eq!(data.len(), 4);

        if !config_address.enable() {
            return;
        }

        let register = config_address.register();
        // let offset = self.config_address.offset();
        let start = register * 4 + offset;

        self.internal.lock().unwrap().handle_ecam_write(
            config_address.bus(),
            config_address.device(),
            config_address.function(),
            start as u16,
            data,
        );
    }

    fn handle_in_config_data(&self, offset: u8, data: &mut [u8]) {
        let config_address = self.config_address.lock().unwrap();

        if !config_address.enable() {
            data.fill(0xff);

            return;
        }

        let register = config_address.register();
        // let offset = self.config_address.offset(); // ignore offset?
        let start = register * 4 + offset;

        self.internal.lock().unwrap().handle_ecam_read(
            config_address.bus(),
            config_address.device(),
            config_address.function(),
            start as u16,
            data,
        );
    }
}

impl Device for PciRootComplexPio {
    fn name(&self) -> String {
        "pci-root-complex".to_string()
    }
}

impl PioDevice for PciRootComplexPio {
    fn ports(&self) -> Vec<PortRange> {
        vec![
            PortRange {
                start: CONFIG_ADDRESS,
                len: 4,
            },
            PortRange {
                start: CONFIG_DATA,
                len: 4,
            },
        ]
    }

    fn io_in(&self, port: u16, data: &mut [u8]) {
        if (CONFIG_ADDRESS..CONFIG_ADDRESS + 4).contains(&port) {
            let offset = port - CONFIG_ADDRESS;
            self.handle_in_config_address(offset as u8, data);
        } else if (CONFIG_DATA..CONFIG_DATA + 4).contains(&port) {
            let offset = port - CONFIG_DATA;
            self.handle_in_config_data(offset as u8, data)
        } else {
            panic!("pci: 0x{:x}", port);
        }
    }

    fn io_out(&self, port: u16, data: &[u8]) {
        if (CONFIG_ADDRESS..CONFIG_ADDRESS + 4).contains(&port) {
            let offset = port - CONFIG_ADDRESS;
            self.handle_out_config_address(offset as u8, data);
        } else if (CONFIG_DATA..CONFIG_DATA + 4).contains(&port) {
            let offset = port - CONFIG_DATA;
            self.handle_out_config_data(offset as u8, data);
        } else {
            panic!("pci: 0x{:x}", port);
        }
    }
}
