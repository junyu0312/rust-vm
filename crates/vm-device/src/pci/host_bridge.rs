use std::collections::HashMap;

use tracing::trace;

use crate::device::pio::PioDevice;
use crate::pci::bus::PciBus;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_DATA: u16 = 0xcfc;

#[derive(Debug)]
struct ConfigAddress {
    enable_bit: bool,
    bus: u8,
    slot: u8,
    func: u8,
    register: u8,
    offset: u8,
}

#[derive(Default)]
pub struct PciHostBridge {
    bus: HashMap<u8, PciBus>,
    last_config_address: Option<ConfigAddress>,
}

impl PioDevice for PciHostBridge {
    fn ports(&self) -> &[u16] {
        &[CONFIG_ADDRESS, CONFIG_DATA]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        trace!(?port, len = data.len());
        match port {
            CONFIG_DATA => {
                for i in data {
                    *i = 0xff;
                }
            }
            _ => unreachable!(),
        }
    }

    fn io_out(&mut self, port: u16, data: &[u8]) {
        match port {
            CONFIG_ADDRESS => {
                assert_eq!(data.len(), 4);
                let data = u32::from_le_bytes(data.try_into().unwrap());
                let enable_bit = (data >> 31) != 0;
                let bus = ((data >> 16) & 0xff) as u8;
                let slot = ((data >> 11) & 0x1f) as u8;
                let func = ((data >> 8) & 0x7) as u8;
                let register = ((data >> 2) & 0x3f) as u8;
                let offset = (register & 0xff) as u8;

                let config_address = ConfigAddress {
                    enable_bit,
                    bus,
                    slot,
                    func,
                    register,
                    offset,
                };
                trace!(?config_address);
                self.last_config_address = Some(config_address);
            }
            _ => unreachable!(),
        }
    }
}
