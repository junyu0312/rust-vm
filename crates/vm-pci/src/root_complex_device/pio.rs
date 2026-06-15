use std::ops::Range;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use vm_core::device::error::DeviceError;
use vm_core::device::pio::pio_device::PioDevice;
use vm_utils::range_allocator::RangeAllocator;

use crate::root_complex::pci_root_complex::PciRootComplex;
use crate::root_complex_device::pio::config_data::ConfigAddress;

mod config_data;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_DATA: u16 = 0xcfc;

pub struct PioTransport {
    pub(crate) io_port_window: Range<u16>,
    config_address: Mutex<ConfigAddress>,
    internal: Arc<RwLock<PciRootComplex>>,
}

impl PioTransport {
    pub fn new(
        pio_allocator: &mut RangeAllocator<u16>,
        io_port_window: Range<u16>,
        internal: Arc<RwLock<PciRootComplex>>,
    ) -> Result<Self, DeviceError> {
        let _ = pio_allocator
            .reserve(CONFIG_ADDRESS, 4)
            .map_err(|_| DeviceError::AllocResource)?;
        let _ = pio_allocator
            .reserve(CONFIG_DATA, 4)
            .map_err(|_| DeviceError::AllocResource)?;
        let _ = pio_allocator
            .reserve(
                io_port_window.start,
                (io_port_window.end - io_port_window.start) as usize,
            )
            .map_err(|_| DeviceError::AllocResource)?;

        Ok(PioTransport {
            io_port_window,
            config_address: Default::default(),
            internal,
        })
    }

    fn handle_out_config_address(&self, offset: u8, data: &[u8]) {
        self.config_address.lock().unwrap().write(offset, data);
    }

    fn handle_in_config_address(&self, offset: u8, data: &mut [u8]) {
        self.config_address.lock().unwrap().read(offset, data);
    }

    fn handle_out_config_data(&self, offset: u8, data: &[u8]) {
        let config_address = self.config_address.lock().unwrap();

        if !config_address.enable() {
            return;
        }

        let register = config_address.register();
        // let offset = self.config_address.offset();
        let start = register * 4 + offset;

        self.internal.read().unwrap().handle_ecam_write(
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

        self.internal.read().unwrap().handle_ecam_read(
            config_address.bus(),
            config_address.device(),
            config_address.function(),
            start as u16,
            data,
        );
    }
}

impl PioDevice for PioTransport {
    fn ports(&self) -> Vec<Range<u16>> {
        vec![
            CONFIG_ADDRESS..CONFIG_ADDRESS + 4,
            CONFIG_DATA..CONFIG_DATA + 4,
            self.io_port_window.clone(),
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
