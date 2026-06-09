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

pub struct VfioBarInfo {
    pub(crate) size: u64,
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

impl PciType0Function for VfioPciFunction {
    fn bar_size(&self) -> [Option<u32>; 6] {
        array::from_fn(|i| {
            self.bars[i]
                .as_ref()
                .map(|bar| bar.size.try_into().unwrap())
        })
    }

    fn bar_handler(&self, _bar: Bar) -> Option<Box<dyn BarHandler>> {
        todo!()
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
