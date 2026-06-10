use std::array;
use std::io::Read;
use std::io::Write;

use vm_core::device::error::DeviceSnapshotError;
use vm_pci::device::function::BarHandler;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::types::configuration_space::ConfigurationSpace;
use vm_pci::types::configuration_space::header::type0::Type0Header;

#[derive(Debug)]
pub enum VfioBarResource {
    Pio,
    Mmio,
}

#[derive(Debug)]
pub struct VfioBarInfo {
    pub(crate) size: u64,
    #[allow(dead_code)]
    pub(crate) resource: VfioBarResource,
}

pub struct VfioPciFunction {
    header: Type0Header,
    bars: [Option<VfioBarInfo>; 6],
}

impl VfioPciFunction {
    pub(crate) fn new(header: Type0Header, bars: [Option<VfioBarInfo>; 6]) -> Self {
        VfioPciFunction { header, bars }
    }
}

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

struct MockBarHandler {}

impl BarHandler for MockBarHandler {
    fn read(&self, _offset: u64, _data: &mut [u8]) {
        todo!()
    }

    fn write(&self, _offset: u64, _data: &[u8]) {
        todo!()
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
