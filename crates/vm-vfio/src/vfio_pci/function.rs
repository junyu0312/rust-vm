use std::io::Read;
use std::io::Write;

use vm_core::device::error::DeviceSnapshotError;
use vm_pci::device::function::BarHandler;
use vm_pci::device::function::PciTypeFunctionCommon;
use vm_pci::device::function::type0::Bar;
use vm_pci::device::function::type0::PciType0Function;
use vm_pci::types::configuration_space::ConfigurationSpace;

pub struct VfioPciFunction;

impl PciTypeFunctionCommon for VfioPciFunction {
    fn vendor_id(&self) -> u16 {
        todo!()
    }

    fn device_id(&self) -> u16 {
        todo!()
    }

    fn class_code(&self) -> u32 {
        todo!()
    }

    fn legacy_interrupt(&self) -> Option<(u8, u8)> {
        todo!()
    }

    fn init_capability(&self, _cfg: &mut ConfigurationSpace) -> Result<(), vm_pci::error::Error> {
        todo!()
    }
}

impl PciType0Function for VfioPciFunction {
    fn bar_size(&self) -> [Option<u32>; 6] {
        todo!()
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
