use std::io::Read;
use std::io::Write;
use std::iter;

use vm_core::device::Device;
use vm_core::device::error::DeviceSnapshotError;
use vm_utils::range_allocator::RangeAllocator;

use crate::device::function::PciTypeFunctionCommon;
use crate::device::function::type0::Bar;
use crate::device::function::type0::PciType0Function;
use crate::device::function::type0::Type0Function;
use crate::error::Error;
use crate::types::bar::PciBarInfo;
use crate::types::configuration_space::ConfigurationSpace;
use crate::types::device::PciDevice;
use crate::types::function::PciFunction;

struct HostBridgeFunction;

impl PciTypeFunctionCommon for HostBridgeFunction {
    fn vendor_id(&self) -> u16 {
        0x1b36 // From qemu log
    }

    fn device_id(&self) -> u16 {
        0x0008 // From qemu log   
    }

    fn class_code(&self) -> u32 {
        0x060000
    }

    fn legacy_interrupt(&self) -> Option<(u8, u8)> {
        None
    }

    fn init_capability(&self, _cfg: &mut ConfigurationSpace) -> Result<(), Error> {
        Ok(())
    }
}

impl PciType0Function for HostBridgeFunction {
    fn bar_info(&self) -> [Option<PciBarInfo>; 6] {
        [None, None, None, None, None, None]
    }

    fn bar_read(&self, _bar: Bar, _offset: u64, _buf: &mut [u8]) {
        unreachable!()
    }

    fn bar_write(&self, _bar: Bar, _offset: u64, _buf: &[u8]) {
        unreachable!()
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }
}

pub struct HostBridgeDevice {
    function: Type0Function<HostBridgeFunction>,
}

impl Device for HostBridgeDevice {
    fn name(&self) -> String {
        "host bridge".to_string()
    }

    fn pause(&self) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn resume(&self) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn save(&self, _writer: &mut dyn Write) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }

    fn load(&mut self, _reader: &mut dyn Read) -> Result<(), DeviceSnapshotError> {
        Ok(())
    }
}

impl PciDevice for HostBridgeDevice {
    fn get_function(&self, function: u8) -> Option<&dyn PciFunction> {
        if function == 0 {
            return Some(&self.function);
        }

        None
    }

    fn get_function_mut(&mut self, function: u8) -> Option<&mut dyn PciFunction> {
        if function == 0 {
            return Some(&mut self.function);
        }

        None
    }

    fn functions(&self) -> Box<dyn Iterator<Item = &(dyn PciFunction + '_)> + '_> {
        Box::new(iter::once(&self.function as &dyn PciFunction))
    }
}

pub fn new_host_bridge(
    #[cfg(target_arch = "x86_64")] pci_pio_allocator: &mut RangeAllocator<u16>,
    pci_mmio_allocator: &mut RangeAllocator<u64>,
) -> Result<HostBridgeDevice, Error> {
    let function = Type0Function::new(
        #[cfg(target_arch = "x86_64")]
        pci_pio_allocator,
        pci_mmio_allocator,
        HostBridgeFunction,
    )?;

    Ok(HostBridgeDevice { function })
}
