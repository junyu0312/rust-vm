use std::collections::HashMap;


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
        &[
            CONFIG_ADDRESS,
            0xcfa,
            0xcfb,
            CONFIG_DATA,
            CONFIG_DATA + 1,
            CONFIG_DATA + 2,
            CONFIG_DATA + 3,
        ]
    }

    fn io_in(&mut self, port: u16, data: &mut [u8]) {
        if port >= 0xcfb && CONFIG_DATA <= CONFIG_DATA + 3 {
            for b in data {
                *b = 0xff;
            }
        }
    }

    fn io_out(&mut self, port: u16, _data: &[u8]) {
        match port {
            CONFIG_ADDRESS => {
                // assert_eq!(data.len(), 4);
                // let data = u32::from_le_bytes(data.try_into().unwrap());
                // let enable_bit = (data >> 31) != 0;
                // let bus = ((data >> 16) & 0xff) as u8;
                // let slot = ((data >> 11) & 0x1f) as u8;
                // let func = ((data >> 8) & 0x7) as u8;
                // let register = ((data >> 2) & 0x3f) as u8;
                // let offset = register;

                // let config_address = ConfigAddress {
                //     enable_bit,
                //     bus,
                //     slot,
                //     func,
                //     register,
                //     offset,
                // };
                // trace!(?config_address);
                // self.last_config_address = Some(config_address);
            }
            0xcfa => {
                println!("{}", port);
            }
            0xcfb => {
                println!("{}", port);
            }
            _ => unreachable!(),
        }
    }
}
