use std::sync::Mutex;

use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::header::type0::Type0Header;
use vm_pci::types::function::EcamUpdateCallback;
use vm_pci::types::function::PciFunction;
use vm_pci::types::function::PciFunctionArch;
use vm_pci::types::function::type0::Type0HeaderOffset;
use vm_pci::types::interrupt::InterruptMapEntry;

#[derive(Debug)]
pub enum VfioBarResource {
    Pio,
    Mmio { is_64bit: bool },
}

#[derive(Debug)]
pub struct VfioBarInfo {
    pub(crate) size: u64,
    #[allow(dead_code)]
    pub(crate) resource: VfioBarResource,
}

pub struct VfioPciFunction {
    configuration_space: Mutex<ConfigurationSpace>,
    bars: [Option<VfioBarInfo>; 6],
}

impl VfioPciFunction {
    pub(crate) fn new(
        configuration_space: ConfigurationSpace,
        bars: [Option<VfioBarInfo>; 6],
    ) -> Self {
        VfioPciFunction {
            configuration_space: configuration_space.into(),
            bars,
        }
    }

    fn write_bar(&self, bar_index: usize, buf: &[u8]) -> Option<EcamUpdateCallback> {
        let mut configuration_space = self.configuration_space.lock().unwrap();
        let header = configuration_space.as_header_mut::<Type0Header>();

        let data = u32::from_le_bytes(buf.try_into().unwrap());

        if data == u32::MAX {
            let size = if let Some(bar_info) = &self.bars[bar_index] {
                bar_info.size as u32
            } else if bar_index > 0
                && let Some(bar_info) = &self.bars[bar_index - 1]
                && let VfioBarResource::Mmio { is_64bit: true } = bar_info.resource
            {
                (bar_info.size >> 32) as u32
            } else {
                0
            };

            header.bar[bar_index] = !(size.wrapping_sub(1));
            None
        } else {
            header.bar[bar_index] = data;
            None
        }
    }
}

impl PciFunctionArch for VfioPciFunction {
    fn interrupt_map_entry(
        &self,
        _bus: u8,
        _device: u8,
        _function: u8,
    ) -> Option<InterruptMapEntry> {
        todo!()
    }
}

fn helper(buf: &[u8]) -> String {
    match buf.len() {
        1 => format!("0x{:x}", buf[0]),
        2 => format!("0x{:x}", u16::from_le_bytes(buf.try_into().unwrap())),
        4 => format!("0x{:x}", u32::from_le_bytes(buf.try_into().unwrap())),
        8 => format!("0x{:x}", u64::from_le_bytes(buf.try_into().unwrap())),
        _ => panic!(),
    }
}
impl PciFunction for VfioPciFunction {
    fn ecam_read(&self, offset: u16, buf: &mut [u8]) {
        self.configuration_space.lock().unwrap().read(offset, buf);

        println!("read offset 0x{:x}: {:?}", offset, helper(buf));
    }

    fn ecam_write(&self, offset: u16, buf: &[u8]) -> Option<EcamUpdateCallback> {
        println!("write offset 0x{:x}: {:?}", offset, helper(buf));

        match Type0HeaderOffset::from_repr(offset) {
            Some(Type0HeaderOffset::Bar0) => self.write_bar(0, buf),
            Some(Type0HeaderOffset::Bar1) => self.write_bar(1, buf),
            Some(Type0HeaderOffset::Bar2) => self.write_bar(2, buf),
            Some(Type0HeaderOffset::Bar3) => self.write_bar(3, buf),
            Some(Type0HeaderOffset::Bar4) => self.write_bar(4, buf),
            Some(Type0HeaderOffset::Bar5) => self.write_bar(5, buf),
            _ => {
                self.configuration_space.lock().unwrap().write(offset, buf);
                None
            }
        }
    }
}
/*
impl PciTypeFunctionCommon for VfioPciFunction {
    fn vendor_id(&self) -> u16 {
        self.header.common.vendor_id
    }

    fn device_id(&self) -> u16 {
        self.header.common.device_id
    }

    fn class_code(&self) -> u32 {
        ((self.header.common.class_code as u32) << 16)
            | ((self.header.common.subclass as u32) << 8)
            | (self.header.common.prog_if as u32)
    }

    fn legacy_interrupt(&self) -> Option<(u8, u8)> {
        None
    }

    fn init_capability(&self, _cfg: &mut ConfigurationSpace) -> Result<(), vm_pci::error::Error> {
        Ok(())
    }
}

impl PciType0Function for VfioPciFunction {
    fn bar_size(&self) -> [Option<u32>; 6] {
        array::from_fn(|i| {
            self.bars[i]
                .as_ref()
                .map(|bar| bar.size.try_into().unwrap())
        })
    }

    fn bar_handler(&self, bar: Bar) -> Option<Box<dyn BarHandler>> {
        match bar {
            Bar::Bar0 => {
                if let Some(_bar) = &self.bars[0] {
                    Some(Box::new(MockBarHandler {}))
                } else {
                    unreachable!()
                }
            }
            Bar::Bar1 => {
                if let Some(_bar) = &self.bars[1] {
                    Some(Box::new(MockBarHandler {}))
                } else {
                    unreachable!()
                }
            }
            Bar::Bar2 => {
                if let Some(_bar) = &self.bars[2] {
                    Some(Box::new(MockBarHandler {}))
                } else {
                    unreachable!()
                }
            }
            Bar::Bar3 => {
                if let Some(_bar) = &self.bars[3] {
                    Some(Box::new(MockBarHandler {}))
                } else {
                    unreachable!()
                }
            }
            Bar::Bar4 => {
                if let Some(_bar) = &self.bars[4] {
                    Some(Box::new(MockBarHandler {}))
                } else {
                    unreachable!()
                }
            }
            Bar::Bar5 => {
                if let Some(_bar) = &self.bars[5] {
                    Some(Box::new(MockBarHandler {}))
                } else {
                    unreachable!()
                }
            }
        }
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        todo!()
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        todo!()
    }
}
*/
